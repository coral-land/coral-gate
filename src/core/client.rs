use kube::config::KubeConfigOptions;

use crate::Result;
use std::{collections::BTreeMap, env, path::PathBuf};

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

fn resolve_kube_path(input_path: &str) -> PathBuf {
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

pub async fn get_kubernetes_client(custom_config_path: &Option<String>) -> Result<kube::Client> {
    match custom_config_path {
        Some(kube_config_path) => {
            let path = resolve_kube_path(kube_config_path);

            // TODO: Error handling
            let kubeconfig = kube::config::Kubeconfig::read_from(path).unwrap();
            let config =
                kube::Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions::default())
                    .await
                    .unwrap();
            let client = kube::Client::try_from(config).unwrap();

            return Ok(client);
        }

        None => return Ok(kube::Client::try_default().await.unwrap()),
    }
}
