use crate::{Result, error::CoralGateError};

use base64::{Engine as _, engine::general_purpose};
use kube::config::KubeConfigOptions;
use std::{env, path::PathBuf, sync::OnceLock};

static KUBE_CLIENT: OnceLock<kube::Client> = OnceLock::new();
static KUBE_CONFIG: OnceLock<kube::Config> = OnceLock::new();

#[derive(Default, Clone)]
pub struct ClientManager {
    config: Option<kube::Config>,
    client: Option<kube::Client>,
}

impl ClientManager {
    pub async fn get_kube_client() -> &'static kube::Client {
        KUBE_CLIENT.get().expect("Kube config is not set")
    }

    pub async fn generate_kube_client(
        &mut self,
        custom_config_path: &Option<String>,
    ) -> Result<kube::Client> {
        let client = match custom_config_path {
            Some(kube_config_path) => {
                let path = self.resolve_kube_path(kube_config_path);
                let kubeconfig = kube::config::Kubeconfig::read_from(path)?;
                let config =
                    kube::Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
                        .await?;

                self.config = Some(config.clone());
                let client = kube::Client::try_from(config)?;
                client
            }

            None => {
                let kubeconfig_options = KubeConfigOptions::default();
                let config = kube::Config::from_kubeconfig(&kubeconfig_options).await?;
                self.config = Some(config.clone());

                kube::Client::try_default().await.unwrap()
            }
        };

        KUBE_CLIENT
            .set(client.clone())
            .unwrap_or_else(|_| panic!("How you doing ? "));

        Ok(client)
    }

    fn resolve_kube_path(&self, input_path: &str) -> PathBuf {
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

    pub fn get_root_cert(&self) -> Result<&Vec<Vec<u8>>> {
        if let Some(config) = &self.config {
            if let Some(root_cert) = config.root_cert.as_ref() {
                return Ok(root_cert);
            } else {
                return Err(CoralGateError::ClientManagerRootCaMissing);
            }
        } else {
            Err(CoralGateError::ClientManagerConfigNotInitialized)
        }
    }

    pub fn root_cert_base64(&self) -> Result<String> {
        let cert_chain = self.get_root_cert()?;
        let mut pem_bundle = String::new();

        for der_cert in cert_chain {
            let b64 = general_purpose::STANDARD.encode(&der_cert);

            pem_bundle.push_str("-----BEGIN CERTIFICATE-----\n");

            for chunk in b64.as_bytes().chunks(64) {
                pem_bundle.push_str(std::str::from_utf8(chunk).unwrap());
                pem_bundle.push('\n');
            }

            pem_bundle.push_str("-----END CERTIFICATE-----\n");
        }

        Ok(general_purpose::STANDARD.encode(pem_bundle))
    }

    pub fn cluster_url(&self) -> Result<String> {
        match &self.config {
            Some(config) => return Ok(config.cluster_url.to_string()),
            None => return Err(CoralGateError::ClientManagerConfigNotInitialized),
        }
    }
}
