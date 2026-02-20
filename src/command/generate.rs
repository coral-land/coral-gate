// ------ Internal
use crate::Result;
use crate::command::structure::GenerateArgs;
use crate::core::client::get_kubernetes_client;
use crate::core::csr;
use crate::error::CoralGateError;

// ----- External
use k8s_openapi::api::certificates::v1::CertificateSigningRequest;

/// Handles the incoming generate command
pub async fn handle(gen_arguments: GenerateArgs) -> Result<()> {
    // TODO: Make the client static
    let client = get_kubernetes_client(&gen_arguments.kubeconfig).await?;
    let self_signed_cert =
        csr::generate_certificate(&gen_arguments.user, &gen_arguments.group).await?;
    let csr_object = csr::generate_cert_sigining_request_object(&gen_arguments, &self_signed_cert)?;
    let csr_api: kube::Api<CertificateSigningRequest> = kube::Api::all(client);

    let created_csr = csr::create(&csr_object, &csr_api).await?;
    let approved_csr = csr::approve(&created_csr, &csr_api).await?;

    let name = approved_csr
        .metadata
        .name
        .ok_or_else(|| CoralGateError::UnknownTempError)?;

    let signed_certificate = csr::get_signed_certificate(&name, &csr_api).await?;

    println!("{signed_certificate:?}");
    Ok(())
}
