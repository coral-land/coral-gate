use clap::{Parser, Subcommand};

// Constants
const DEFAULT_KUBECONFIG: &str = "~/.kube/config";
const DEFAULT_VALIDITY_HOURS: i32 = 24 * 30;

#[derive(Debug, Clone, clap::Args)]
pub struct CommonArgs {}

macro_rules! define_args {
    (
        $vis:vis struct $name:ident {
            $(
                $(#[$attr:meta])*
                $vis_field:vis $field:ident: $ty:ty
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, clap::Args)]
        $vis struct $name {
            #[arg(long, default_value = DEFAULT_KUBECONFIG)]
            pub kubeconfig: Option<String>,

            #[arg(short, long)]
            pub namespace: Option<String>,

            $(
                $(#[$attr])*
                $vis_field $field: $ty,
            )*
        }
    };
}

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

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PermissionProfile {
    ClusterReadonly,
    Admin,
}

impl PermissionProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionProfile::Admin => "admin",
            PermissionProfile::ClusterReadonly => "cluster-readonly",
        }
    }
}

define_args! {
    pub struct SetupArgs { }
}

define_args! {
    pub struct GenerateArgs {
        /// Username to create
        #[arg(short, long)]
        pub user: String,

        #[arg(short, long, default_value = "./kubeconfig")]
        output: String,

        /// How long the kubeconfig should be valid (in hours)
        #[arg(short, long, default_value_t = DEFAULT_VALIDITY_HOURS)]
        pub expire: i32,

        /// Predefined policies (Admin, Readonly)
        #[arg(short, long)]
        pub profile: PermissionProfile,
    }
}
