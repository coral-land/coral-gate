use std::sync::OnceLock;

use crate::command::structure::{Cli, PermissionProfile};

#[derive(Clone)]
pub struct GateInformation {
    pub user: Option<String>,
    pub group: Option<String>,
    pub namespace: Option<String>,
    pub profile: Option<PermissionProfile>,
    pub command: Option<Cli>,

    client: Option<kube::Client>,
}

static GATE_INFORMATION: OnceLock<GateInformation> = OnceLock::new();

pub fn init_gate_config(config: GateInformation) {
    GATE_INFORMATION
        .set(config)
        .unwrap_or_else(|_| panic!("Config not initialized"))
}

pub fn get_gate_config() -> &'static GateInformation {
    GATE_INFORMATION
        .get()
        .unwrap_or_else(|| panic!("Can not get the gate config"))
}
