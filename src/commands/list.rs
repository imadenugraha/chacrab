use anyhow::{Context, Result};

use crate::storage::{get_session_key, list_all_credentials, Database};

/// List all credentials (labels and URLs only, no decryption)
pub async fn list_credentials(db: &Database) -> Result<()> {
    let pool = db.pool();

    // Verify session (though we don't need the key for listing)
    get_session_key()
        .context("Not logged in. Please run 'chacrab login' first.")?;

    // Fetch all credentials
    let credentials = list_all_credentials(pool).await?;

    if credentials.is_empty() {
        println!("📭 No credentials stored yet");
        println!("   Use 'chacrab add' to add your first credential");
        return Ok(());
    }

    println!("\n📝 Stored Credentials ({})\n", credentials.len());
    println!("{:<30} {:<40} Created", "Label", "URL");
    println!("{}", "─".repeat(85));

    for cred in credentials {
        let url = cred.url.as_deref().unwrap_or("-");
        let created = cred.created_at.format("%Y-%m-%d %H:%M");
        println!("{:<30} {:<40} {}", cred.label, url, created);
    }

    println!();

    Ok(())
}
