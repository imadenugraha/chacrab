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
#[command(after_help = "EXAMPLES:
  # Initialize a new vault
  chacrab init

  # Log in to unlock your vault
  chacrab login

  # Add a credential interactively
  chacrab add

  # Add a credential with flags
  chacrab add --label GitHub --username octocat --password s3cr3t --url https://github.com

  # Get a credential (copies to clipboard)
  chacrab get --label GitHub

  # List all credentials
  chacrab list

  # Update an existing credential
  chacrab update --label GitHub

  # Delete a credential
  chacrab delete --label GitHub
  chacrab rm --label OldAccount    # 'rm' is an alias for delete

  # Log out to clear your session
  chacrab logout

NOTES:
  - All credentials are encrypted with ChaCha20-Poly1305
  - Master password is never stored, only derived with Argon2id
  - Session keys are stored securely in your OS keyring
  - For more help on a command: chacrab <command> --help")]
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
    #[command(long_about = "Initialize a new ChaCrab vault with your master password.\n\nThis creates a new database and generates a random salt for key derivation.\nYour master password will be used to encrypt all credentials using Argon2id + ChaCha20-Poly1305.\n\n⚠️  Important: There is NO way to recover your data if you forget your master password!")]
    Init,

    /// Login to unlock the vault
    #[command(long_about = "Authenticate with your master password to unlock the vault.\n\nThis derives your encryption key from the master password and stores it securely\nin your OS keyring for the current session. You'll remain logged in until you\nrun 'chacrab logout' or restart your system.")]
    Login,

    /// Logout and clear session
    #[command(long_about = "Clear your active session and remove the encryption key from your OS keyring.\n\nAfter logging out, you'll need to run 'chacrab login' again to access your credentials.")]
    Logout,

    /// Add a new credential
    #[command(long_about = "Add a new credential to your vault.\n\nYou can provide values via flags or be prompted interactively.\nEach credential is encrypted with a unique nonce before storage.\n\nExample:\n  chacrab add --label GitHub --username octocat --password s3cr3t\n  chacrab add  # Interactive mode")]
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
    #[command(long_about = "Retrieve and decrypt a credential from your vault.\n\nBy default, the password is copied to your clipboard for security.\nYou can choose to display it in the terminal instead.\n\nExample:\n  chacrab get --label GitHub\n  chacrab get  # Will prompt for label")]
    Get {
        /// Label of the credential to retrieve
        #[arg(short, long)]
        label: Option<String>,
    },

    /// List all stored credentials
    #[command(alias = "ls")]
    #[command(long_about = "Display all stored credentials (labels and URLs only).\n\nPasswords are NOT decrypted or displayed in this view.\nUse 'chacrab get' to retrieve and decrypt a specific credential.\n\nAlias: 'ls'")]
    List,

    /// Delete a credential
    #[command(alias = "rm")]
    #[command(long_about = "Permanently delete a credential from your vault.\n\nThis action cannot be undone. The credential is immediately removed from the database.\n\nExample:\n  chacrab delete --label OldAccount\n  chacrab rm --label GitHub  # 'rm' is an alias\n\nAlias: 'rm'")]
    Delete {
        /// Label of the credential to delete
        #[arg(short, long)]
        label: Option<String>,
    },

    /// Update an existing credential
    #[command(long_about = "Update an existing credential in your vault.\n\nYou can update username, password, and/or URL. In interactive mode,\nyou'll be prompted to select which fields to change.\n\nExample:\n  chacrab update --label GitHub\n  chacrab update --label GitHub --password newP@ssw0rd")]
    Update {
        /// Label of the credential to update
        #[arg(short, long)]
        label: Option<String>,

        /// New username
        #[arg(short, long)]
        username: Option<String>,

        /// New password
        #[arg(short, long)]
        password: Option<String>,

        /// New URL
        #[arg(long)]
        url: Option<String>,
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
        Commands::Update {
            label,
            username,
            password,
            url,
        } => commands::update_credential_cmd(&db, label, username, password, url).await?,
    }

    Ok(())
}

