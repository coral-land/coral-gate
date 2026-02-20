use crate::{command::structure::GenerateArgs, error::*};
use std::{collections::BTreeMap, env, path::PathBuf};

use k8s_openapi::{
    ByteString,
    api::certificates::v1::{
        CertificateSigningRequest as K8SCertificateSigningRequest, CertificateSigningRequestSpec,
    },
    apimachinery::pkg::apis::meta::v1::ObjectMeta,
};

use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, PKCS_RSA_SHA256, RsaKeySize};
use tokio::time::{self, sleep};

const DEFAULT_RETRIE_COUNT: i32 = 32;

pub struct GeneratedCsrWithPem {
    csr: rcgen::CertificateSigningRequest,
    key_pair: rcgen::KeyPair,
    key_pem: String,
}

pub async fn generate_certificate(user: &str, group: &str) -> Result<GeneratedCsrWithPem> {
    let key_pair = KeyPair::generate_rsa_for(&PKCS_RSA_SHA256, RsaKeySize::_2048).unwrap();
    let key_pem = key_pair.serialize_pem();

    let mut distinguished_name = DistinguishedName::new();
    let mut params = CertificateParams::default();

    distinguished_name.push(DnType::OrganizationName, group);
    distinguished_name.push(DnType::CommonName, user);

    params.distinguished_name = distinguished_name;
    params.subject_alt_names = vec![];

    let csr = params.serialize_request(&key_pair).unwrap();

    Ok(GeneratedCsrWithPem {
        csr,
        key_pair,
        key_pem,
    })
}

/// TODO: Move to shared or utils
pub fn generate_lables(
    user: &String,
    group: &String,
) -> Option<BTreeMap<std::string::String, std::string::String>> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();

    labels.insert("created-by".into(), "coralgate".into());
    labels.insert("user".into(), user.into());
    labels.insert("group".into(), group.into());

    Some(labels)
}

/*
* Generate Certificate Signing Request (csr) object
* The certificates are automatically generated
*/
pub fn generate_cert_sigining_request_object(
    gen_args: &GenerateArgs,
    certificates: &GeneratedCsrWithPem,
) -> Result<K8SCertificateSigningRequest> {
    let labels = generate_lables(&gen_args.user, &gen_args.group);
    let pem_string = certificates.csr.pem().unwrap();
    let request = ByteString(pem_string.into_bytes());

    let metadata = ObjectMeta {
        name: Some(format!("{}-csr", gen_args.user)),
        labels: labels,
        ..Default::default()
    };

    let spec = CertificateSigningRequestSpec {
        request,
        signer_name: "kubernetes.io/kube-apiserver-client".into(),
        usages: Some(vec!["client auth".to_string()]),
        expiration_seconds: Some(gen_args.expire * 3600),
        ..Default::default()
    };

    let signing_request_object = K8SCertificateSigningRequest {
        metadata,
        spec,
        ..Default::default()
    };

    Ok(signing_request_object)
}

// TODO: Move to utils
pub fn resolve_kube_path(input_path: &str) -> PathBuf {
    if input_path.starts_with('~') {
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .expect("Could not find home directory environment variable");

        let mut path = PathBuf::from(home);
        let stripped = input_path
            .strip_prefix("~/")
            .unwrap_or(input_path.strip_prefix("~").unwrap_or(""));
        path.push(stripped);
        path
    } else {
        PathBuf::from(input_path)
    }
}

pub async fn create(
    csr_object: &K8SCertificateSigningRequest,
    api: &kube::Api<K8SCertificateSigningRequest>,
) -> Result<K8SCertificateSigningRequest> {
    let created_csr = api
        .create(&kube::api::PostParams::default(), &csr_object)
        .await
        .unwrap();

    Ok(created_csr)
}

pub async fn approve(
    created_csr: &K8SCertificateSigningRequest,
    csr_api: &kube::Api<K8SCertificateSigningRequest>,
) -> Result<K8SCertificateSigningRequest> {
    let csr_name = created_csr.metadata.name.as_ref().unwrap();
    let approval_patch = serde_json::json!({
        "apiVersion": "certificates.k8s.io/v1",
        "kind": "CertificateSigningRequest",
        "status": {
            "conditions": [{
                "type": "Approved",
                "status": "True",
                "reason": "coralgate auto approval, cli usage",
                "message": "Approved by coralgate tool",
                "lastUpdateTime": chrono::Utc::now().to_rfc3339(),
            }]
        }
    });

    let approved_csr = csr_api
        .patch_approval(
            csr_name,
            &kube::api::PatchParams::default(),
            &kube::api::Patch::Merge(approval_patch),
        )
        .await
        .unwrap();

    Ok(approved_csr)
}

/// Returns Byte String sined certificate
pub async fn get_signed_certificate(
    name: &String,
    csr_api: &kube::Api<K8SCertificateSigningRequest>,
) -> Result<ByteString> {
    for i in 0..DEFAULT_RETRIE_COUNT {
        if let Ok(csr) = csr_api.get(name).await {
            if let Some(status) = &csr.status {
                if let Some(certificate) = &status.certificate {
                    return Ok(certificate.clone());
                }
            }
            sleep(time::Duration::from_secs(1)).await;
        }
    }

    sleep(time::Duration::from_secs(30)).await;
    Err(CoralGateError::TimeoutError(
        "Timed out getting signed certificate".into(),
    ))
}
