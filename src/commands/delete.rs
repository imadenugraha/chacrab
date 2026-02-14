use anyhow::{Context, Result};
use dialoguer::{Confirm, Input};

use crate::storage::{delete_credential_by_label, get_session_key, Database};
use crate::ui::is_test_mode;

/// Delete a credential
pub async fn delete_credential(db: &Database, label: Option<String>) -> Result<()> {
    let pool = db.pool();

    // Get session key (verify logged in)
    get_session_key()
        .context("Not logged in. Please run 'chacrab login' first.")?;

    // Prompt for label if not provided
    let label = if let Some(l) = label {
        l
    } else {
        Input::new()
            .with_prompt("Label")
            .interact_text()
            .context("Failed to read label")?
    };

    // Confirm deletion
    let confirmed = if is_test_mode() {
        true
    } else {
        Confirm::new()
            .with_prompt(format!("Delete credential '{}'?", label))
            .default(false)
            .interact()
            .context("Failed to read confirmation")?
    };

    if !confirmed {
        println!("Cancelled");
        return Ok(());
    }

    // Delete from database
    let deleted = delete_credential_by_label(pool, &label)
        .await
        .context("Failed to delete credential")?;

    if deleted {
        println!("✅ Credential '{}' deleted successfully", label);
    } else {
        anyhow::bail!(
            "Credential '{}' not found.\n   Use 'chacrab list' to see all stored credentials.",
            label
        );
    }

    Ok(())
}
