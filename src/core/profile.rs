use crate::error::*;

use k8s_openapi::api::rbac::v1::{ClusterRoleBinding, RoleBinding, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client};

#[async_trait::async_trait]
pub trait Apply {
    async fn apply(&self, client: &kube::Client) -> Result<()>;
}

#[derive(Default)]
pub struct Profile {
    name: String,
    pub resources: Vec<Box<dyn Apply + Send + Sync>>,
}

impl Profile {
    pub fn add_resource(&mut self, resource: Box<dyn Apply + Send + Sync>) {
        self.resources.push(resource);
    }

    pub async fn apply(&self, client: &kube::Client) -> Result<()> {
        for resource in &self.resources {
            resource.apply(client).await?
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Apply for ClusterRoleBinding {
    async fn apply(&self, client: &Client) -> Result<()> {
        let api: Api<ClusterRoleBinding> = Api::all(client.clone());

        let name = self
            .metadata
            .name
            .as_ref()
            .ok_or(CoralGateError::MissingName("Cluster Role binding".into()))?;

        api.patch(
            name,
            &PatchParams::apply("kaccess").force(),
            &Patch::Apply(self),
        )
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Apply for RoleBinding {
    async fn apply(&self, client: &Client) -> Result<()> {
        let namespace = self
            .metadata
            .namespace
            .as_ref()
            .ok_or(CoralGateError::MissingNamespace("Role binding".into()))?;

        let api: Api<RoleBinding> = Api::namespaced(client.clone(), namespace);
        let name = self
            .metadata
            .name
            .as_ref()
            .ok_or(CoralGateError::MissingName("Role Binding".into()))?;

        api.patch(
            name,
            &PatchParams::apply("kaccess").force(),
            &Patch::Apply(self),
        )
        .await?;

        Ok(())
    }
}

pub fn admin_profile() -> Profile {
    let binding = ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some("cluster-admin-binding".into()),
            ..Default::default()
        },
        subjects: Some(vec![Subject {
            kind: "Group".into(),
            name: "cluster-admins".into(), // Certificates must use O=cluster-admins
            api_group: Some("rbac.authorization.k8s.io".into()),
            namespace: None,
        }]),
        role_ref: RoleRef {
            kind: "ClusterRole".into(),
            name: "cluster-admin".into(),
            api_group: "rbac.authorization.k8s.io".into(),
        },
    };

    Profile {
        name: "admin".into(),
        resources: vec![Box::new(binding)],
    }
}

pub fn cluster_readonly_profile() -> Profile {
    let binding = ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some("cluster-readonly-binding".into()),
            ..Default::default()
        },
        subjects: Some(vec![Subject {
            kind: "Group".into(),
            name: "cluster-readonly".into(),
            api_group: Some("rbac.authorization.k8s.io".into()),
            namespace: None,
        }]),
        role_ref: RoleRef {
            kind: "ClusterRole".into(),
            name: "view".into(),
            api_group: "rbac.authorization.k8s.io".into(),
        },
    };

    Profile {
        name: "cluster-readonly".into(),
        resources: vec![Box::new(binding)],
    }
}

pub fn namespaced_readonly(namespace: &str) -> Profile {
    let binding = RoleBinding {
        metadata: ObjectMeta {
            name: Some(format!("readonly-{}", namespace)),
            namespace: Some(namespace.into()),
            ..Default::default()
        },
        subjects: Some(vec![Subject {
            kind: "Group".into(),
            name: format!("readonly-{}", namespace),
            api_group: Some("rbac.authorization.k8s.io".into()),
            namespace: None,
        }]),
        role_ref: RoleRef {
            kind: "ClusterRole".into(),
            name: "view".into(),
            api_group: "rbac.authorization.k8s.io".into(),
        },
    };

    Profile {
        name: format!("readonly-{}", namespace),
        resources: vec![Box::new(binding)],
    }
}
