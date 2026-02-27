use clap::{ArgGroup, Args, Parser, Subcommand};

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
    Update {
        #[command(subcommand)]
        target: UpdateCommands,
    },
    BackupExport { path: String },
    BackupImport { path: String },
    Sync,
    Config,
}

#[derive(Debug, Subcommand)]
pub enum UpdateCommands {
    Password(UpdateSelector),
    #[command(name = "secret-notes")]
    SecretNotes(UpdateSelector),
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("selector")
        .required(true)
        .args(["id", "label"])
))]
pub struct UpdateSelector {
    #[arg(long)]
    pub id: Option<String>,
    #[arg(long)]
    pub label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands, UpdateCommands};
    use clap::Parser;

    #[test]
    fn parses_update_password_id() {
        let cli = Cli::try_parse_from(["chacrab", "update", "password", "--id", "abc123"])
            .expect("command should parse");

        let Commands::Update { target } = cli.command else {
            panic!("expected update command");
        };
        let UpdateCommands::Password(selector) = target else {
            panic!("expected password update");
        };
        assert_eq!(selector.id.as_deref(), Some("abc123"));
        assert!(selector.label.is_none());
    }

    #[test]
    fn parses_update_secret_notes_label() {
        let cli = Cli::try_parse_from([
            "chacrab",
            "update",
            "secret-notes",
            "--label",
            "Personal note",
        ])
        .expect("command should parse");

        let Commands::Update { target } = cli.command else {
            panic!("expected update command");
        };
        let UpdateCommands::SecretNotes(selector) = target else {
            panic!("expected secret-notes update");
        };
        assert_eq!(selector.label.as_deref(), Some("Personal note"));
        assert!(selector.id.is_none());
    }
}
