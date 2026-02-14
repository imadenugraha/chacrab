use anyhow::{Context, Result};

use crate::storage::{delete_session_key, has_session_key};

/// Logout and clear session key from keyring
pub async fn logout() -> Result<()> {
    if !has_session_key() {
        println!("ℹ️  Not logged in");
        return Ok(());
    }

    delete_session_key()
        .context("Failed to delete session key")?;

    println!("✅ Logged out successfully");

    Ok(())
}
