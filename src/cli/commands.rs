use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use std::{fs, time::Duration};
use uuid::Uuid;
use zeroize::Zeroize;

use crate::{
    auth::login,
    cli::{
        display::{
            SessionIndicator, UiOptions, clear_screen, configure_terminal, error as error_msg,
            is_insecure_terminal, print_header, secure, short_id, success, syncing, system,
            warning,
        },
        parser::{Cli, Commands},
        prompts, runtime_config, session, table,
    },
    core::{
        backup::{EncryptedBackupFile, export_encrypted, import_encrypted},
        errors::{ChacrabError, ChacrabResult},
        models::VaultItem,
        password_policy,
        vault::VaultService,
    },
    storage::{app::AppRepository, r#trait::VaultRepository},
    sync::sync_engine::SyncEngine,
};

async fn app_repo(cli: &Cli) -> ChacrabResult<AppRepository> {
    let repo = AppRepository::connect(&cli.backend, &cli.database_url).await?;
    repo.init().await?;
    Ok(repo)
}

fn ui_options(cli: &Cli) -> UiOptions {
    UiOptions {
        json: cli.json,
        quiet: cli.quiet,
        color: !cli.no_color,
    }
}

fn map_user_error(err: &ChacrabError) -> &'static str {
    match err {
        ChacrabError::InvalidCredentials => "Invalid master password.",
        ChacrabError::NoActiveSession => "No active session. Run login first.",
        ChacrabError::SessionExpired => "Session timed out. Please login again.",
        ChacrabError::NotFound => "Item not found.",
        ChacrabError::UnsupportedBackend(_) => "Unsupported backend configuration.",
        ChacrabError::Config(message) if message == "operation cancelled" => "Operation cancelled.",
        ChacrabError::Config(message) if message == "ambiguous item id prefix" => {
            "Ambiguous ID. Use a longer ID prefix."
        }
        ChacrabError::Config(message) if message.starts_with("weak master password") => {
            "Weak master password. Use at least 12 chars and 3 of upper/lower/digit/symbol."
        }
        ChacrabError::Config(message) if message == "confirmation text did not match title" => {
            "Confirmation text did not match title."
        }
        ChacrabError::Config(_) => "Invalid configuration or input.",
        ChacrabError::KeyringLocked => "Secure keyring is locked. Unlock your keyring and retry.",
        ChacrabError::KeyringUnavailable => "Secure keyring unavailable. Unlock keyring and retry.",
        ChacrabError::Crypto => "Security operation failed.",
        ChacrabError::Serialization => "Data format error.",
        ChacrabError::Storage => "Storage operation failed.",
    }
}

fn parse_or_resolve_id(id_input: &str, items: &[VaultItem]) -> ChacrabResult<Uuid> {
    if let Ok(id) = Uuid::parse_str(id_input) {
        return Ok(id);
    }

    let mut matches = items
        .iter()
        .filter(|item| item.id.to_string().starts_with(id_input))
        .map(|item| item.id);

    let Some(first) = matches.next() else {
        return Err(ChacrabError::NotFound);
    };

    if matches.next().is_some() {
        return Err(ChacrabError::Config("ambiguous item id prefix".to_owned()));
    }

    Ok(first)
}

pub async fn run() -> ChacrabResult<()> {
    let mut cli = Cli::parse();
    let args = std::env::args().collect::<Vec<_>>();
    let backend_explicit = runtime_config::cli_flag_present(&args, "--backend");
    let database_url_explicit = runtime_config::cli_flag_present(&args, "--database-url");

    if (!backend_explicit || !database_url_explicit)
        && let Some(saved_config) = runtime_config::load()?
    {
        if !backend_explicit {
            cli.backend = saved_config.backend;
        }
        if !database_url_explicit {
            cli.database_url = saved_config.database_url;
        }
    }

    let options = ui_options(&cli);
    configure_terminal(options.color);

    if is_insecure_terminal() {
        warning(
            "Insecure terminal detected (output redirected). Secret reveal is disabled.",
            options,
        );
    }

    let repo = app_repo(&cli).await?;
    let vault = VaultService::new(repo.clone());

    let session_indicator = match session::session_state() {
        session::SessionState::Active => SessionIndicator::Active,
        session::SessionState::Locked => SessionIndicator::Locked,
    };

    let result = match &cli.command {
        Commands::Init => run_init(&repo, &cli, options, session_indicator).await,
        Commands::Login => run_login(&repo, &cli, options, session_indicator).await,
        Commands::Logout => run_logout(options, session_indicator),
        Commands::AddPassword => run_add_password(&vault, &cli, options, session_indicator).await,
        Commands::AddNote => run_add_note(&vault, &cli, options, session_indicator).await,
        Commands::List => run_list(&vault, &cli, options, session_indicator).await,
        Commands::Show { id } => run_show(&vault, &cli, options, session_indicator, id).await,
        Commands::Delete { id } => run_delete(&vault, &cli, options, session_indicator, id).await,
        Commands::BackupExport { path } => {
            run_backup_export(&vault, &cli, options, session_indicator, path).await
        }
        Commands::BackupImport { path } => {
            run_backup_import(&vault, &cli, options, session_indicator, path).await
        }
        Commands::Sync => run_sync(&vault, &cli, options, session_indicator).await,
        Commands::Config => run_config(&cli, options, session_indicator),
    };

    if let Err(err) = &result {
        error_msg(map_user_error(err), options);
    }

    result
}

async fn run_init(
    repo: &AppRepository,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Chacrab Vault Initialization", session_indicator, options);
    secure("Create master password:", options);
    let password = prompts::secure_password_with_confirmation(
        "Master password: ",
        "Confirm master password: ",
    )?;
    password_policy::validate_master_password(password.expose_secret())?;

    warning("This password cannot be recovered.", options);
    let proceed = prompts::confirmation_prompt("Proceed?", false)?;
    if !proceed {
        return Err(ChacrabError::Config("operation cancelled".to_owned()));
    }

    login::register(repo, password).await?;
    let vault_id = repo
        .get_auth_record()
        .await?
        .map(|record| short_id(&record.salt))
        .unwrap_or_else(|| "local".to_owned());

    success("Vault initialized successfully.", options);
    system(&format!("Vault ID: {vault_id}"), options);
    system(
        &format!("Storage: {}", backend_display(&cli.backend)),
        options,
    );

    runtime_config::save(&runtime_config::RuntimeConfig {
        backend: cli.backend.clone(),
        database_url: cli.database_url.clone(),
    })?;

    Ok(())
}

async fn run_login(
    repo: &AppRepository,
    _cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Chacrab Login", session_indicator, options);
    secure("Enter master password:", options);
    let password = prompts::secure_password_prompt("Master password: ")?;
    login::login(repo, password).await?;
    session::touch_session()?;
    success("Login successful.", options);
    system("Session: active", options);
    Ok(())
}

fn run_logout(options: UiOptions, session_indicator: SessionIndicator) -> ChacrabResult<()> {
    print_header("Chacrab Logout", session_indicator, options);
    secure("Terminating session...", options);
    login::logout()?;
    session::clear_session_metadata()?;
    success("Vault locked.", options);
    Ok(())
}

async fn run_add_password(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Add New Credential", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let title = prompts::input("Title")?;
    let username = prompts::optional_input("Username/Email")?;
    let url = prompts::optional_input("URL")?;
    let password = prompts::secure_password_prompt("Password: ")?;
    let notes = prompts::multiline("Notes (optional multiline)")?;

    let mut key = login::current_session_key()?;
    let item = vault
        .add_password(title, username, url, password, notes, &key)
        .await?;
    key.zeroize();
    session::touch_session()?;

    success("Credential stored securely.", options);
    system(&format!("ID: {}", short_id(&item.id.to_string())), options);
    Ok(())
}

async fn run_add_note(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Add Secure Note", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let title = prompts::input("Title")?;
    let note = prompts::multiline("Content (multiline)")?.unwrap_or_default();

    let mut key = login::current_session_key()?;
    vault
        .add_note(title, SecretString::new(note.into_boxed_str()), &key)
        .await?;
    key.zeroize();
    session::touch_session()?;

    success("Secure note stored.", options);
    Ok(())
}

async fn run_list(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Vault Items", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let items = vault.list().await?;
    session::touch_session()?;

    if options.json {
        let output = items
            .iter()
            .map(|item| {
                json!({
                    "id": short_id(&item.id.to_string()),
                    "type": format!("{:?}", item.r#type).to_lowercase(),
                    "title": item.title,
                    "updated": item.updated_at.format("%Y-%m-%d").to_string()
                })
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            serde_json::to_string(&output).map_err(|_| ChacrabError::Serialization)?
        );
        return Ok(());
    }

    table::print_list_table(&items);
    Ok(())
}

async fn run_show(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
    id: &str,
) -> ChacrabResult<()> {
    print_header("Credential Details", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let all_items = vault.list().await?;
    let resolved_id = parse_or_resolve_id(id, &all_items)?;

    let mut key = login::current_session_key()?;
    let (item, payload) = vault.show_decrypted(resolved_id, &key).await?;
    key.zeroize();
    session::touch_session()?;

    let username = item.username.clone().unwrap_or_else(|| "-".to_owned());
    let url = item.url.clone().unwrap_or_else(|| "-".to_owned());
    let mut password = payload
        .get("password")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_owned();

    if options.json {
        let out = json!({
            "id": short_id(&item.id.to_string()),
            "type": format!("{:?}", item.r#type).to_lowercase(),
            "title": item.title,
            "username": username,
            "url": url,
            "password": "********"
        });
        println!(
            "{}",
            serde_json::to_string(&out).map_err(|_| ChacrabError::Serialization)?
        );
        password.zeroize();
        return Ok(());
    }

    system(&format!("Title: {}", item.title), options);
    system(&format!("Username: {username}"), options);
    system(&format!("URL: {url}"), options);
    system("Password: ********", options);

    if is_insecure_terminal() {
        warning(
            "Sensitive actions are blocked on insecure terminal output.",
            options,
        );
        password.zeroize();
        return Ok(());
    }

    let choice = prompts::select("Options", &["Reveal password", "Copy to clipboard", "Exit"])?;

    match choice {
        0 => {
            if is_insecure_terminal() {
                warning("Reveal blocked on insecure terminal.", options);
            } else if password.is_empty() {
                warning("No password stored for this item.", options);
            } else {
                system(&format!("Password: {}", password), options);
                warning("Password will clear in 10 seconds.", options);
                tokio::time::sleep(Duration::from_secs(10)).await;
                clear_screen(options);
                system("Password view cleared.", options);
            }
        }
        1 => {
            if is_insecure_terminal() {
                warning("Clipboard copy blocked on insecure terminal.", options);
            } else if password.is_empty() {
                warning("No password stored for this item.", options);
            } else {
                let mut clipboard = arboard::Clipboard::new()
                    .map_err(|_| ChacrabError::Config("clipboard unavailable".to_owned()))?;
                clipboard
                    .set_text(password.clone())
                    .map_err(|_| ChacrabError::Config("clipboard write failed".to_owned()))?;
                success(
                    "Password copied. Clearing clipboard in 15 seconds.",
                    options,
                );
                tokio::time::sleep(Duration::from_secs(15)).await;
                let _ = clipboard.set_text(String::new());
                system("Clipboard cleared.", options);
            }
        }
        _ => {}
    }

    password.zeroize();
    Ok(())
}

async fn run_delete(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
    id: &str,
) -> ChacrabResult<()> {
    print_header("Delete Item", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let all_items = vault.list().await?;
    let resolved_id = parse_or_resolve_id(id, &all_items)?;
    let item = vault.repository().get_item(resolved_id).await?;

    warning("Are you sure you want to delete this item?", options);
    let typed = prompts::input("Type the title to confirm")?;
    if typed != item.title {
        return Err(ChacrabError::Config(
            "confirmation text did not match title".to_owned(),
        ));
    }

    vault.delete(resolved_id).await?;
    session::touch_session()?;
    success("Item deleted permanently.", options);
    Ok(())
}

async fn run_sync(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Sync", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    syncing("Syncing encrypted vault...", options);
    let local_count = vault.list().await?.len() as u64;
    let remote = sync_remote_repo().await?;
    let remote_count = remote.list_items().await?.len() as u64;
    let total = (local_count + remote_count).max(1);

    if !options.json && !options.quiet {
        let progress = ProgressBar::new(total.max(1));
        progress.set_style(
            ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len}")
                .map_err(|_| ChacrabError::Config("invalid progress style".to_owned()))?,
        );
        for _ in 0..total.max(1) {
            progress.inc(1);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        progress.finish_and_clear();
    }

    let report = SyncEngine::sync_bidirectional(vault.repository(), &remote).await?;
    session::touch_session()?;
    success("Sync complete.", options);
    system(&format!("Items uploaded: {}", report.uploaded), options);
    system(&format!("Items downloaded: {}", report.downloaded), options);
    if report.conflicts > 0 {
        let short_ids = report
            .conflict_ids
            .iter()
            .take(5)
            .map(|id| id.to_string().chars().take(8).collect::<String>())
            .collect::<Vec<_>>()
            .join(", ");
        warning(
            &format!(
                "⚠️ Sync conflicts resolved: {} ({short_ids})",
                report.conflicts
            ),
            options,
        );
    }
    if report.replay_blocked > 0 {
        warning(
            &format!(
                "⚠️ Replay-protection blocks: {} stale remote update(s) ignored",
                report.replay_blocked
            ),
            options,
        );
    }
    Ok(())
}

async fn sync_remote_repo() -> ChacrabResult<AppRepository> {
    let backend = std::env::var("CHACRAB_SYNC_BACKEND")
        .map_err(|_| ChacrabError::Config("set CHACRAB_SYNC_BACKEND for sync".to_owned()))?;
    let database_url = std::env::var("CHACRAB_SYNC_DATABASE_URL")
        .map_err(|_| ChacrabError::Config("set CHACRAB_SYNC_DATABASE_URL for sync".to_owned()))?;
    validate_sync_remote_config(&backend, &database_url)?;

    let repo = AppRepository::connect(&backend, &database_url).await?;
    repo.init().await?;
    Ok(repo)
}

fn validate_sync_remote_config(backend: &str, database_url: &str) -> ChacrabResult<()> {
    let normalized_backend = backend.trim().to_ascii_lowercase();
    let require_tls = std::env::var("CHACRAB_SYNC_REQUIRE_TLS")
        .map(|value| value != "0" && !value.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    match normalized_backend.as_str() {
        "sqlite" => {}
        "postgres" => {
            let lowered = database_url.to_ascii_lowercase();
            if !(lowered.starts_with("postgres://") || lowered.starts_with("postgresql://")) {
                return Err(ChacrabError::Config(
                    "sync postgres URL must start with postgres:// or postgresql://".to_owned(),
                ));
            }
            if require_tls
                && !(lowered.contains("sslmode=require")
                    || lowered.contains("sslmode=verify-ca")
                    || lowered.contains("sslmode=verify-full"))
            {
                return Err(ChacrabError::Config(
                    "sync postgres URL must enable TLS (sslmode=require|verify-ca|verify-full)"
                        .to_owned(),
                ));
            }
        }
        "mongo" => {
            let lowered = database_url.to_ascii_lowercase();
            if !(lowered.starts_with("mongodb://") || lowered.starts_with("mongodb+srv://")) {
                return Err(ChacrabError::Config(
                    "sync mongo URL must start with mongodb:// or mongodb+srv://".to_owned(),
                ));
            }
            if require_tls
                && lowered.starts_with("mongodb://")
                && !(lowered.contains("tls=true") || lowered.contains("ssl=true"))
            {
                return Err(ChacrabError::Config(
                    "sync mongo URL must enable TLS (tls=true)".to_owned(),
                ));
            }
        }
        _ => {
            return Err(ChacrabError::Config(
                "sync backend must be sqlite, postgres, or mongo".to_owned(),
            ));
        }
    }

    if normalized_backend != "sqlite" {
        let token = std::env::var("CHACRAB_SYNC_AUTH_TOKEN").map_err(|_| {
            ChacrabError::Config("set CHACRAB_SYNC_AUTH_TOKEN for remote sync auth".to_owned())
        })?;
        if token.trim().len() < 16 {
            return Err(ChacrabError::Config(
                "CHACRAB_SYNC_AUTH_TOKEN must be at least 16 characters".to_owned(),
            ));
        }
    }

    Ok(())
}

async fn run_backup_export(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
    path: &str,
) -> ChacrabResult<()> {
    print_header("Encrypted Backup Export", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let mut key = login::current_session_key()?;
    let items = vault.list().await?;
    let backup = export_encrypted(items.clone(), &key)?;
    key.zeroize();

    let serialized = serde_json::to_string_pretty(&backup)?;
    fs::write(path, serialized).map_err(|_| ChacrabError::Storage)?;
    session::touch_session()?;

    success("Encrypted backup exported.", options);
    system(&format!("Path: {path}"), options);
    system(&format!("Items exported: {}", items.len()), options);
    Ok(())
}

async fn run_backup_import(
    vault: &VaultService<AppRepository>,
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
    path: &str,
) -> ChacrabResult<()> {
    print_header("Encrypted Backup Import", session_indicator, options);
    session::enforce_timeout(cli.session_timeout_secs)?;

    let content = fs::read_to_string(path).map_err(|_| ChacrabError::Storage)?;
    let backup_file: EncryptedBackupFile = serde_json::from_str(&content)?;

    let mut key = login::current_session_key()?;
    let payload = import_encrypted(&backup_file, &key)?;
    key.zeroize();

    let repo = vault.repository();
    for item in &payload.items {
        repo.upsert_item(item).await?;
    }
    session::touch_session()?;

    success("Encrypted backup imported.", options);
    system(&format!("Items imported: {}", payload.items.len()), options);
    Ok(())
}

fn run_config(
    cli: &Cli,
    options: UiOptions,
    session_indicator: SessionIndicator,
) -> ChacrabResult<()> {
    print_header("Configuration", session_indicator, options);
    if options.json {
        let value = json!({
            "backend": cli.backend,
            "database_url": cli.database_url,
            "json": cli.json,
            "quiet": cli.quiet,
            "no_color": cli.no_color,
            "session_timeout_secs": cli.session_timeout_secs,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|_| ChacrabError::Serialization)?
        );
    } else {
        system(
            &format!("Backend: {}", backend_display(&cli.backend)),
            options,
        );
        system(&format!("Database URL: {}", cli.database_url), options);
        system(&format!("JSON mode: {}", cli.json), options);
        system(&format!("Quiet mode: {}", cli.quiet), options);
        system(&format!("Color disabled: {}", cli.no_color), options);
        system(
            &format!("Session timeout (sec): {}", cli.session_timeout_secs),
            options,
        );
    }
    Ok(())
}

fn backend_display(backend: &str) -> &'static str {
    match backend {
        "sqlite" => "SQLite (local)",
        "postgres" => "PostgreSQL",
        "mongo" => "MongoDB",
        _ => "Unsupported",
    }
}
