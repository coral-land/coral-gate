use crate::{
    command::structure::SetupArgs,
    core::{
        client::ClientManager,
        profile::{admin_profile, cluster_readonly_profile},
    },
    error::*,
};

/// We will apply default roles and rolebindings
pub async fn handle(arguments: SetupArgs) -> Result<()> {
    let mut client_manager = ClientManager::default();
    let admin = admin_profile();
    let cluster_readonly = cluster_readonly_profile();

    let client = client_manager
        .generate_kube_client(&arguments.kubeconfig)
        .await?;

    admin.apply(&client).await?;
    cluster_readonly.apply(&client).await?;

    Ok(())
}
