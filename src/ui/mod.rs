// UI utilities module
// Most UI interaction is handled directly in commands via dialoguer
// This module can be extended with shared display/formatting functions

pub mod password_validator;

pub use password_validator::{validate_password, StrengthLevel};

pub fn is_test_mode() -> bool {
    std::env::var("CHACRAB_TEST_MODE")
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

pub fn test_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[allow(dead_code)]
pub fn print_banner() {
    println!("🦀 ChaCrab - Zero-Knowledge Password Manager");
    println!();
}
