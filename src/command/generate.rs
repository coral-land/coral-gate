use crate::Result;
use crate::command::structure::GenerateArgs;
use crate::core::client::ClientManager;
use crate::core::csr;
use crate::error::CoralGateError;

use base64::Engine;
use base64::engine::general_purpose;
use k8s_openapi::api::certificates::v1::CertificateSigningRequest;
use tokio::fs;

/// TODO: Create a generator, give the options to it and then call generate
pub async fn handle(gen_arguments: GenerateArgs) -> Result<()> {
    let mut client_manager = ClientManager::default();
    let client = client_manager
        .generate_kube_client(&gen_arguments.kubeconfig)
        .await?;

    let self_signed_cert =
        csr::generate_certificate(&gen_arguments.user, gen_arguments.profile.as_str()).await?;

    let csr_object = csr::generate_cert_sigining_request_object(&gen_arguments, &self_signed_cert)?;
    let csr_api: kube::Api<CertificateSigningRequest> = kube::Api::all(client);

    let created_csr = csr::create(&csr_object, &csr_api).await?;
    let approved_csr = csr::approve(&created_csr, &csr_api).await?;

    let name = approved_csr
        .metadata
        .name
        .ok_or_else(|| CoralGateError::UnknownTempError)?;

    let signed_cert = csr::get_signed_certificate(&name, &csr_api).await?;

    let signed_cert_b64 = general_purpose::STANDARD.encode(&signed_cert.0);
    let private_key_b64 = general_purpose::STANDARD.encode(self_signed_cert.key_pem.as_bytes());

    let root_ca_b64 = client_manager.root_cert_base64()?;
    let cluster_url = client_manager.cluster_url()?;

    let kubeconfig_yaml = format!(
        r#"apiVersion: v1
kind: Config
clusters:
- cluster:
    certificate-authority-data: {root_ca}
    server: {cluster_url}
  name: cluster-default
contexts:
- context:
    cluster: cluster-default
    user: {user}
  name: {user}-context
current-context: {user}-context
users:
- name: {user}
  user:
    client-certificate-data: {cert}
    client-key-data: {key}
"#,
        root_ca = root_ca_b64,
        cluster_url = cluster_url,
        user = gen_arguments.user,
        cert = signed_cert_b64,
        key = private_key_b64
    );

    fs::write("kubeconfig", kubeconfig_yaml.as_bytes()).await;

    Ok(())
}
