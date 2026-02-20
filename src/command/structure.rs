use clap::{Parser, Subcommand};

// Constants
const DEFAULT_KUBECONFIG: &str = "~/.kube/config";
const DEFAULT_VALIDITY_HOURS: i32 = 24 * 30;

#[derive(Parser, Debug, Clone)]
#[command(version, about = "This is a package for you to create temporary safe kubeconfigs", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Generates a kubeconfig with restricted access
    Generate(GenerateArgs),

    /// Setups predefined roles and role bindings used to issue kubeconfig.
    /// This needs admin access
    Setup(SetupArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct SetupArgs {}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PermissionProfile {
    ClusterReadonly,
    NamespacedReadonly,
    Admin,
}

#[derive(Parser, Debug, Clone)]
pub struct GenerateArgs {
    /// Username to create
    #[arg(short, long)]
    pub user: String,

    /// Group to assign user
    #[arg(short, long)]
    pub group: String,

    /// Restrict access to a namespace
    #[arg(short, long)]
    pub namespace: Option<String>,

    /// Path to master kubeconfig or one that has privilege to control RBAC
    #[arg(long, default_value = DEFAULT_KUBECONFIG)]
    pub kubeconfig: Option<String>,

    /// How long the kubeconfig should be valid (in hours)
    #[arg(short, long, default_value_t = DEFAULT_VALIDITY_HOURS)]
    pub expire: i32,

    /// Predefined policies (Admin, Readonly)
    #[arg(short, long)]
    pub profile: PermissionProfile,
}
