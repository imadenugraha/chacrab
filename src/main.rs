mod commands;
mod crypto;
mod models;
mod storage;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use storage::init_db;

#[derive(Parser)]
#[command(name = "chacrab")]
#[command(about = "🦀 ChaCrab - Zero-Knowledge Password Manager", long_about = None)]
#[command(version)]
struct Cli {
    /// Database URL (defaults to sqlite://chacrab.db or DATABASE_URL env var)
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite://chacrab.db")]
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault with a master password
    Init,

    /// Login to unlock the vault
    Login,

    /// Logout and clear session
    Logout,

    /// Add a new credential
    Add {
        /// Label for the credential (e.g., "GitHub")
        #[arg(short, long)]
        label: Option<String>,

        /// Username or email
        #[arg(short, long)]
        username: Option<String>,

        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Optional URL
        #[arg(long)]
        url: Option<String>,
    },

    /// Get and display a credential
    Get {
        /// Label of the credential to retrieve
        #[arg(short, long)]
        label: Option<String>,
    },

    /// List all stored credentials
    #[command(alias = "ls")]
    List,

    /// Delete a credential
    #[command(alias = "rm")]
    Delete {
        /// Label of the credential to delete
        #[arg(short, long)]
        label: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    // Initialize database connection
    let db = init_db(&cli.database).await?;

    // Execute command
    match cli.command {
        Commands::Init => commands::init_vault(&db).await?,
        Commands::Login => commands::login(&db).await?,
        Commands::Logout => commands::logout().await?,
        Commands::Add {
            label,
            username,
            password,
            url,
        } => commands::add_credential(&db, label, username, password, url).await?,
        Commands::Get { label } => commands::get_credential(&db, label).await?,
        Commands::List => commands::list_credentials(&db).await?,
        Commands::Delete { label } => commands::delete_credential(&db, label).await?,
    }

    Ok(())
}

