use crate::core::errors::{ChacrabError, ChacrabResult};

const MIN_MASTER_PASSWORD_LENGTH: usize = 12;

pub fn validate_master_password(candidate: &str) -> ChacrabResult<()> {
    if candidate.chars().count() < MIN_MASTER_PASSWORD_LENGTH {
        return Err(ChacrabError::Config(
            "weak master password: use at least 12 characters".to_owned(),
        ));
    }

    let has_lower = candidate.chars().any(|ch| ch.is_ascii_lowercase());
    let has_upper = candidate.chars().any(|ch| ch.is_ascii_uppercase());
    let has_digit = candidate.chars().any(|ch| ch.is_ascii_digit());
    let has_symbol = candidate
        .chars()
        .any(|ch| !ch.is_ascii_alphanumeric() && !ch.is_whitespace());

    let class_count = [has_lower, has_upper, has_digit, has_symbol]
        .into_iter()
        .filter(|present| *present)
        .count();

    if class_count < 3 {
        return Err(ChacrabError::Config(
            "weak master password: use at least 3 of upper/lower/digit/symbol".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_master_password;

    #[test]
    fn rejects_short_password() {
        assert!(validate_master_password("Aa1!short").is_err());
    }

    #[test]
    fn rejects_low_complexity_password() {
        assert!(validate_master_password("alllowercase12").is_err());
        assert!(validate_master_password("UPPERCASEONLY12").is_err());
    }

    #[test]
    fn accepts_valid_password() {
        assert!(validate_master_password("StrongPass12!").is_ok());
    }
}
