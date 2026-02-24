use clap::{Parser, Subcommand};

pub const DEFAULT_BACKEND: &str = "sqlite";
pub const DEFAULT_DATABASE_URL: &str = "sqlite://chacrab.db?mode=rwc";

#[derive(Debug, Parser)]
#[command(
    name = "chacrab",
    version,
    about = "Security-first CLI password manager"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, default_value = DEFAULT_BACKEND)]
    pub backend: String,

    #[arg(long, default_value = DEFAULT_DATABASE_URL)]
    pub database_url: String,

    #[arg(long, default_value_t = false, global = true)]
    pub json: bool,

    #[arg(long, default_value_t = false, global = true)]
    pub quiet: bool,

    #[arg(long, default_value_t = false, global = true)]
    pub no_color: bool,

    #[arg(long, default_value_t = 900, global = true)]
    pub session_timeout_secs: u64,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Login,
    Logout,
    AddPassword,
    AddNote,
    List,
    Show { id: String },
    Delete { id: String },
    BackupExport { path: String },
    BackupImport { path: String },
    Sync,
    Config,
}
