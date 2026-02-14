//! Password strength validation module
//! 
//! Provides comprehensive password quality checking including:
//! - Length validation (minimum 12 characters recommended)
//! - Complexity checks (uppercase, lowercase, numbers, symbols)
//! - Entropy calculation
//! - Common password detection

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrengthLevel {
    Weak,       // 0-40: High risk
    Fair,       // 41-60: Moderate risk
    Strong,     // 61-80: Low risk
    Excellent,  // 81-100: Very low risk
}

impl StrengthLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            StrengthLevel::Weak => "Weak",
            StrengthLevel::Fair => "Fair",
            StrengthLevel::Strong => "Strong",
            StrengthLevel::Excellent => "Excellent",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            StrengthLevel::Weak => "⚠️ ",
            StrengthLevel::Fair => "💡",
            StrengthLevel::Strong => "✅",
            StrengthLevel::Excellent => "🎉",
        }
    }
}

#[derive(Debug)]
pub struct PasswordStrength {
    #[allow(dead_code)]
    pub score: u8,
    pub level: StrengthLevel,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Common passwords list (top 50 most common)
const COMMON_PASSWORDS: &[&str] = &[
    "password", "123456", "123456789", "12345678", "12345", "1234567",
    "password1", "123123", "1234567890", "000000", "abc123", "qwerty",
    "iloveyou", "monkey", "dragon", "111111", "letmein", "sunshine",
    "princess", "admin", "welcome", "login", "solo", "1234", "starwars",
    "qwertyuiop", "passw0rd", "password123", "master", "hello",
    "freedom", "whatever", "trustno1", "jordan", "654321",
    "superman", "batman", "michael", "jennifer", "football",
    "charlie", "shadow", "baseball", "hunter", "thomas", "killer",
    "tigger", "robert", "nicole", "secret",
];

/// Calculate Shannon entropy of password
fn calculate_entropy(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }

    let mut char_counts: std::collections::HashMap<char, usize> = std::collections::HashMap::new();
    for ch in password.chars() {
        *char_counts.entry(ch).or_insert(0) += 1;
    }

    let len = password.len() as f64;
    let mut entropy = 0.0;

    for count in char_counts.values() {
        let probability = (*count as f64) / len;
        entropy -= probability * probability.log2();
    }

    entropy * len
}

/// Check if password contains sequential characters
fn has_sequential_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    for window in chars.windows(3) {
        // Check ascending sequence (abc, 123)
        if (window[1] as u32 == window[0] as u32 + 1)
            && (window[2] as u32 == window[1] as u32 + 1)
        {
            return true;
        }
        // Check descending sequence (cba, 321)
        if window[0] as u32 > 1
            && (window[1] as u32 == window[0] as u32 - 1)
            && (window[2] as u32 == window[1] as u32 - 1)
        {
            return true;
        }
    }

    false
}

/// Check if password has repeated characters
fn has_repeated_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < 3 {
        return false;
    }

    for window in chars.windows(3) {
        if window[0] == window[1] && window[1] == window[2] {
            return true;
        }
    }

    false
}

/// Check if password is a common password
fn is_common_password(password: &str) -> bool {
    let lowercase = password.to_lowercase();
    COMMON_PASSWORDS.contains(&lowercase.as_str())
}

/// Validate password strength and return detailed analysis
pub fn validate_password(password: &str) -> PasswordStrength {
    let mut score = 0u8;
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // Length check (0-25 points)
    let len = password.len();
    if len < 8 {
        warnings.push("Password is too short (minimum 8 characters)".to_string());
    } else if len < 12 {
        warnings.push("Password is shorter than recommended 12 characters".to_string());
        score += (len as u8 - 8) * 3; // 0-12 points for 8-11 chars
    } else if len < 16 {
        score += 15; // 15 points for 12-15 chars
    } else {
        score += 25; // 25 points for 16+ chars
    }

    // Complexity checks (0-60 points total)
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    let mut complexity_score = 0;
    if has_lowercase {
        complexity_score += 15;
    } else {
        suggestions.push("Add lowercase letters (a-z)".to_string());
    }

    if has_uppercase {
        complexity_score += 15;
    } else {
        suggestions.push("Add uppercase letters (A-Z)".to_string());
    }

    if has_digit {
        complexity_score += 15;
    } else {
        suggestions.push("Add numbers (0-9)".to_string());
    }

    if has_special {
        complexity_score += 15;
    } else {
        suggestions.push("Add special characters (!@#$%^&*)".to_string());
    }

    score += complexity_score;

    // Entropy check (0-15 points)
    let entropy = calculate_entropy(password);
    if entropy < 30.0 {
        warnings.push("Low entropy - password is predictable".to_string());
    } else if entropy < 50.0 {
        score += 7;
    } else {
        score += 15;
    }

    // Pattern detection (penalties)
    if is_common_password(password) {
        warnings.push("This is a commonly used password - easily guessed".to_string());
        score = score.saturating_sub(40);
    }

    if has_sequential_chars(password) {
        warnings.push("Contains sequential characters (e.g., abc, 123)".to_string());
        score = score.saturating_sub(10);
    }

    if has_repeated_chars(password) {
        warnings.push("Contains repeated characters (e.g., aaa, 111)".to_string());
        score = score.saturating_sub(10);
    }

    // Character set diversity bonus
    let char_set: HashSet<char> = password.chars().collect();
    let diversity_ratio = char_set.len() as f64 / len as f64;
    if diversity_ratio < 0.5 {
        warnings.push("Too many repeated characters".to_string());
    }

    // Determine level based on final score
    let level = match score {
        0..=40 => StrengthLevel::Weak,
        41..=60 => StrengthLevel::Fair,
        61..=80 => StrengthLevel::Strong,
        _ => StrengthLevel::Excellent,
    };

    // Add general suggestions for weak passwords
    if (level == StrengthLevel::Weak || level == StrengthLevel::Fair) && suggestions.is_empty() {
        suggestions.push("Consider using a passphrase (4+ random words)".to_string());
        suggestions.push("Avoid personal information (names, dates, etc.)".to_string());
    }

    PasswordStrength {
        score,
        level,
        warnings,
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_very_weak_password() {
        let strength = validate_password("123");
        assert_eq!(strength.level, StrengthLevel::Weak);
        assert!(strength.score < 20);
    }

    #[test]
    fn test_common_password() {
        let strength = validate_password("password");
        assert_eq!(strength.level, StrengthLevel::Weak);
        assert!(strength.warnings.iter().any(|w| w.contains("commonly used")));
    }

    #[test]
    fn test_weak_password() {
        let strength = validate_password("12345678");
        assert_eq!(strength.level, StrengthLevel::Weak);
        assert!(!strength.warnings.is_empty());
    }

    #[test]
    fn test_fair_password() {
        let strength = validate_password("simplepass123");
        assert!(matches!(
            strength.level,
            StrengthLevel::Weak | StrengthLevel::Fair | StrengthLevel::Strong
        ));
    }

    #[test]
    fn test_strong_password() {
        let strength = validate_password("MyP@ssw0rd123!");
        assert!(matches!(
            strength.level,
            StrengthLevel::Strong | StrengthLevel::Excellent
        ));
        assert!(strength.score >= 60);
    }

    #[test]
    fn test_excellent_password() {
        let strength = validate_password("Tr0ub4dor&3lonG!pass");
        assert!(matches!(
            strength.level,
            StrengthLevel::Strong | StrengthLevel::Excellent
        ));
        assert!(strength.score >= 70);
    }

    #[test]
    fn test_passphrase() {
        let strength = validate_password("correct-horse-battery-staple");
        assert!(matches!(
            strength.level,
            StrengthLevel::Strong | StrengthLevel::Excellent
        ));
    }

    #[test]
    fn test_sequential_detection() {
        let strength = validate_password("abc12345");
        assert!(strength.warnings.iter().any(|w| w.contains("sequential")));
    }

    #[test]
    fn test_repeated_chars_detection() {
        let strength = validate_password("aaabbbccc");
        assert!(strength.warnings.iter().any(|w| w.contains("repeated")));
    }

    #[test]
    fn test_entropy_calculation() {
        let entropy1 = calculate_entropy("aaaaaaaaaa");
        let entropy2 = calculate_entropy("aAbBcCdDeE");
        assert!(entropy2 > entropy1);
    }

    #[test]
    fn test_minimum_length() {
        let strength = validate_password("Short1!");
        assert!(strength.warnings.iter().any(|w| w.contains("short")));
    }

    #[test]
    fn test_suggestions_present() {
        let strength = validate_password("alllowercase");
        assert!(!strength.suggestions.is_empty());
        assert!(strength
            .suggestions
            .iter()
            .any(|s| s.contains("uppercase") || s.contains("numbers")));
    }
}
