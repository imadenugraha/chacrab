use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;

/// Helper to get a test database URL
fn test_db_url() -> (NamedTempFile, String) {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let db_url = format!("sqlite://{}", temp_file.path().display());
    (temp_file, db_url)
}

/// Helper to create a ChaCrab command with the test database
fn chacrab_cmd(db_url: &str) -> Command {
    let mut cmd = Command::cargo_bin("chacrab").expect("Failed to find chacrab binary");
    cmd.arg("--database").arg(db_url);
    cmd
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
        .stdout(predicate::str::contains("Session cleared"));

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
        .write_stdin("wrongpassword\n")
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
    let mut cmd = Command::cargo_bin("chacrab").expect("Failed to find chacrab binary");
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"))
        .stdout(predicate::str::contains("chacrab init"));

    // Test command-specific help
    let mut cmd = Command::cargo_bin("chacrab").expect("Failed to find chacrab binary");
    cmd.arg("add")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a new credential"));
}
