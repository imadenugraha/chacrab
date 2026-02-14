use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

fn test_master_password() -> String {
    std::env::var("CHACRAB_TEST_MASTER_PASSWORD")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "testpass123".to_string())
}

fn postgres_test_url() -> Option<String> {
    std::env::var("CHACRAB_POSTGRES_TEST_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn keyring_namespace(db_url: &str) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    db_url.hash(&mut hasher);
    format!("chacrab-test-{}", hasher.finish())
}

/// Helper to get a test database URL
fn test_db_url() -> (NamedTempFile, String) {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let db_url = format!("sqlite://{}", temp_file.path().display());
    (temp_file, db_url)
}

/// Helper to create a ChaCrab command with the test database
fn chacrab_cmd(db_url: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chacrab"));
    cmd.arg("--database").arg(db_url);
    cmd.env("CHACRAB_TEST_MODE", "1");
    cmd.env("CHACRAB_MASTER_PASSWORD", test_master_password());
    cmd.env("CHACRAB_IMPORT_DUPLICATE", "skip");
    cmd.env("CHACRAB_KEYRING_SERVICE", keyring_namespace(db_url));
    cmd.env("CHACRAB_KEYRING_USERNAME", "test");
    cmd
}

#[test]
fn test_postgres_real_instance_workflow() {
    let Some(db_url) = postgres_test_url() else {
        eprintln!("Skipping PostgreSQL integration test: CHACRAB_POSTGRES_TEST_URL is not set");
        return;
    };

    if !db_url.starts_with("postgres://") && !db_url.starts_with("postgresql://") {
        panic!("CHACRAB_POSTGRES_TEST_URL must start with postgres:// or postgresql://");
    }

    let label_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let label = format!("PgValidation{}", label_suffix);

    // Initialize vault (allow idempotent reuse if already initialized)
    let init_output = {
        let mut cmd = chacrab_cmd(&db_url);
        cmd.arg("init")
            .output()
            .expect("Failed to run init for PostgreSQL validation")
    };

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        assert!(
            stderr.contains("already initialized"),
            "Unexpected init failure: {}",
            stderr
        );
    }

    // Ensure an authenticated session
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("login").assert().success();

    // Add credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg(&label)
        .arg("--username").arg("pg_user")
        .arg("--password").arg("pg_pass_123")
        .arg("--url").arg("https://postgres.example")
        .assert()
        .success();

    // List and verify presence
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains(&label));

    // Update
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("update")
        .arg("--label").arg(&label)
        .arg("--password").arg("pg_pass_456")
        .assert()
        .success();

    // Delete
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("delete")
        .arg("--label").arg(&label)
        .assert()
        .success();

    // Logout/Login smoke check
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout").assert().success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("login").assert().success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout").assert().success();
}

#[test]
fn test_full_workflow() {
    let (_temp, db_url) = test_db_url();
    
    // Step 1: Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpassword123\ntestpassword123\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Vault initialized successfully"));

    // Step 2: Logout
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout")
        .assert()
        .success()
        .stdout(predicate::str::contains("Logged out successfully"));

    // Step 3: Login
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("login")
        .write_stdin("testpassword123\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Logged in successfully"));

    // Step 4: Add credential with flags (non-interactive)
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("TestAccount")
        .arg("--username").arg("testuser")
        .arg("--password").arg("testpass123")
        .arg("--url").arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("Credential 'TestAccount' added successfully"));

    // Step 5: List credentials
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("TestAccount"))
        .stdout(predicate::str::contains("https://example.com"));

    // Step 6: Add another credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("GitHub")
        .arg("--username").arg("octocat")
        .arg("--password").arg("ghp_secret123")
        .assert()
        .success();

    // Step 7: Update credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("update")
        .arg("--label").arg("GitHub")
        .arg("--password").arg("newpassword456")
        .assert()
        .success()
        .stdout(predicate::str::contains("Credential 'GitHub' updated successfully"));

    // Step 8: Get credential (non-interactive not fully supported due to dialoguer)
    // This test is limited as get command uses interactive Select
    // In real usage, this would display the credential

    // Step 9: Delete credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("delete")
        .arg("--label").arg("TestAccount")
        .write_stdin("yes\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Credential 'TestAccount' deleted successfully"));

    // Step 10: Verify deletion by listing
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("GitHub"))
        .stdout(predicate::str::contains("TestAccount").not());

    // Step 11: Logout
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout")
        .assert()
        .success();
}

#[test]
fn test_wrong_password_on_login() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("correctpassword\ncorrectpassword\n")
        .assert()
        .success();

    // Logout
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout")
        .assert()
        .success();

    // Try to login with wrong password
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("login")
        .env("CHACRAB_MASTER_PASSWORD", "wrongpassword")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Incorrect master password"));
}

#[test]
fn test_duplicate_label_error() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize and login
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // Add first credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("DuplicateTest")
        .arg("--username").arg("user1")
        .arg("--password").arg("pass1")
        .assert()
        .success();

    // Try to add duplicate label
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("DuplicateTest")
        .arg("--username").arg("user2")
        .arg("--password").arg("pass2")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_get_nonexistent_credential() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // Try to get non-existent credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("get")
        .arg("--label").arg("NonExistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_commands_without_session() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // Logout
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout")
        .assert()
        .success();

    // Try to add without being logged in
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("Test")
        .arg("--username").arg("user")
        .arg("--password").arg("pass")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not logged in"));

    // Try to list without being logged in
    // Note: list might work differently depending on implementation
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not logged in"));

    // Try to get without being logged in
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("get")
        .arg("--label").arg("Test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not logged in"));

    // Try to delete without being logged in
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("delete")
        .arg("--label").arg("Test")
        .write_stdin("yes\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not logged in"));

    // Try to update without being logged in
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("update")
        .arg("--label").arg("Test")
        .arg("--password").arg("newpass")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not logged in"));
}

#[test]
fn test_update_nonexistent_credential() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // Try to update non-existent credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("update")
        .arg("--label").arg("NonExistent")
        .arg("--password").arg("newpass")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_init_twice_fails() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault first time
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // Try to initialize again
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_list_empty_vault() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // List credentials (should be empty)
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No credentials stored"));
}

#[test]
fn test_command_aliases() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass\ntestpass\n")
        .assert()
        .success();

    // Add a credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("TestAlias")
        .arg("--username").arg("user")
        .arg("--password").arg("pass")
        .assert()
        .success();

    // Test 'ls' alias for list
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("TestAlias"));

    // Test 'rm' alias for delete
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("rm")
        .arg("--label").arg("TestAlias")
        .write_stdin("yes\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted successfully"));
}

#[test]
fn test_help_displays_examples() {
    // Test main help
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chacrab"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"))
        .stdout(predicate::str::contains("chacrab init"));

    // Test command-specific help
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_chacrab"));
    cmd.arg("add")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a new credential"));
}

#[test]
fn test_export_empty_vault() {
    let (_temp, db_url) = test_db_url();
    let export_file = NamedTempFile::new().expect("Failed to create export file");
    let export_path = export_file.path().to_str().unwrap();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass123\ntestpass123\n")
        .assert()
        .success();

    // Export empty vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("export")
        .arg("--output").arg(export_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No credentials to export"));
}

#[test]
fn test_export_and_import_round_trip() {
    let (_temp, db_url) = test_db_url();
    let export_file = NamedTempFile::new().expect("Failed to create export file");
    let export_path = export_file.path().to_str().unwrap();
    
    // Initialize vault and add credentials
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass123\ntestpass123\n")
        .assert()
        .success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("GitHub")
        .arg("--username").arg("octocat")
        .arg("--password").arg("ghp_secret123")
        .arg("--url").arg("https://github.com")
        .assert()
        .success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("Gmail")
        .arg("--username").arg("test@gmail.com")
        .arg("--password").arg("gmail_pass456")
        .assert()
        .success();

    // Export credentials
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("export")
        .arg("--output").arg(export_path)
        .write_stdin("y\n") // Overwrite confirmation
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported 2 credential(s)"));

    // Delete one credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("delete")
        .arg("--label").arg("Gmail")
        .write_stdin("yes\n")
        .assert()
        .success();

    // Import credentials back (should handle duplicate for GitHub)
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("import")
        .arg("--input").arg(export_path)
        .write_stdin("0\n") // Skip GitHub duplicate
        .assert()
        .success()
        .stdout(predicate::str::contains("Import Summary"));

    // Verify both credentials exist
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("GitHub"))
        .stdout(predicate::str::contains("Gmail"));
}

#[test]
fn test_import_with_duplicates() {
    let (_temp, db_url) = test_db_url();
    let export_file = NamedTempFile::new().expect("Failed to create export file");
    let export_path = export_file.path().to_str().unwrap();
    
    // Initialize and add credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass123\ntestpass123\n")
        .assert()
        .success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("TestCred")
        .arg("--username").arg("user1")
        .arg("--password").arg("pass1")
        .assert()
        .success();

    // Export
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("export")
        .arg("--output").arg(export_path)
        .write_stdin("y\n")
        .assert()
        .success();

    // Update the credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("update")
        .arg("--label").arg("TestCred")
        .arg("--password").arg("updated_pass")
        .assert()
        .success();

    // Import with overwrite option (choice 1)
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("import")
        .arg("--input").arg(export_path)
        .env("CHACRAB_IMPORT_DUPLICATE", "overwrite")
        .assert()
        .success()
        .stdout(predicate::str::contains("Overwritten"));
}

#[test]
fn test_import_rename_duplicate() {
    let (_temp, db_url) = test_db_url();
    let export_file = NamedTempFile::new().expect("Failed to create export file");
    let export_path = export_file.path().to_str().unwrap();
    
    // Initialize and add credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass123\ntestpass123\n")
        .assert()
        .success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("Original")
        .arg("--username").arg("user1")
        .arg("--password").arg("pass1")
        .assert()
        .success();

    // Export
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("export")
        .arg("--output").arg(export_path)
        .write_stdin("y\n")
        .assert()
        .success();

    // Import with rename option (choice 2)
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("import")
        .arg("--input").arg(export_path)
        .env("CHACRAB_IMPORT_DUPLICATE", "rename")
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported as"));

    // Verify both exist
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Original"));
}

#[test]
fn test_export_file_permissions() {
    let (_temp, db_url) = test_db_url();
    let export_file = NamedTempFile::new().expect("Failed to create export file");
    let export_path = export_file.path().to_str().unwrap();
    
    // Initialize and add credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("testpass123\ntestpass123\n")
        .assert()
        .success();

    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("Test")
        .arg("--username").arg("user")
        .arg("--password").arg("pass")
        .assert()
        .success();

    // Export
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("export")
        .arg("--output").arg(export_path)
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("File permissions set to 0600"));

    // Verify file exists
    assert!(std::path::Path::new(export_path).exists());
}

#[test]
fn test_password_change_verification() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("oldpassword123\noldpassword123\n")
        .assert()
        .success();

    // Add a credential
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("add")
        .arg("--label").arg("TestAccount")
        .arg("--username").arg("user")
        .arg("--password").arg("pass123")
        .assert()
        .success();

    // Logout
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("logout")
        .assert()
        .success();

    // Try change password with wrong current password
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("change-password")
        .env("CHACRAB_CURRENT_PASSWORD", "wrongpassword")
        .env("CHACRAB_NEW_PASSWORD", "NewStrongPassword123!")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Current password is incorrect"));
}

#[test]
fn test_password_change_requires_fair_strength() {
    let (_temp, db_url) = test_db_url();
    
    // Initialize vault
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("init")
        .write_stdin("StrongPassword123!\nStrongPassword123!\n")
        .assert()
        .success();

    // Try to change to weak password
    let mut cmd = chacrab_cmd(&db_url);
    cmd.arg("change-password")
        .env("CHACRAB_CURRENT_PASSWORD", "testpass123")
        .env("CHACRAB_NEW_PASSWORD", "weak")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Password too weak"));
}
