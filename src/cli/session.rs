use chrono::Utc;

use crate::{
    auth::login,
    core::{errors::{ChacrabError, ChacrabResult}},
};

const SESSION_META_SERVICE: &str = "chacrab";
const SESSION_META_USER: &str = "session-last-activity";

#[derive(Debug, Clone, Copy)]
pub enum SessionState {
    Active,
    Locked,
}

pub fn session_state() -> SessionState {
    if login::current_session_key().is_ok() {
        SessionState::Active
    } else {
        SessionState::Locked
    }
}

pub fn touch_session() -> ChacrabResult<()> {
    let entry = ::keyring::Entry::new(SESSION_META_SERVICE, SESSION_META_USER)?;
    entry.set_password(&Utc::now().timestamp().to_string())?;
    Ok(())
}

pub fn clear_session_metadata() -> ChacrabResult<()> {
    let entry = ::keyring::Entry::new(SESSION_META_SERVICE, SESSION_META_USER)?;
    let _ = entry.delete_password();
    Ok(())
}

pub fn enforce_timeout(timeout_secs: u64) -> ChacrabResult<()> {
    let _ = login::current_session_key()?;

    let entry = ::keyring::Entry::new(SESSION_META_SERVICE, SESSION_META_USER)?;
    let stored = entry.get_password().ok();

    let Some(timestamp_str) = stored else {
        touch_session()?;
        return Ok(());
    };

    let Ok(last) = timestamp_str.as_str().parse::<i64>() else {
        clear_session_metadata()?;
        return Err(ChacrabError::SessionExpired);
    };

    let now = Utc::now().timestamp();
    if is_expired(now, last, timeout_secs) {
        login::logout()?;
        clear_session_metadata()?;
        return Err(ChacrabError::SessionExpired);
    }

    touch_session()?;
    Ok(())
}

fn is_expired(now: i64, last: i64, timeout_secs: u64) -> bool {
    now.saturating_sub(last) > timeout_secs as i64
}

#[cfg(test)]
mod tests {
    use super::is_expired;

    #[test]
    fn expires_when_age_exceeds_timeout() {
        assert!(is_expired(1_000, 900, 50));
    }

    #[test]
    fn does_not_expire_on_timeout_boundary() {
        assert!(!is_expired(1_000, 900, 100));
    }

    #[test]
    fn does_not_expire_before_timeout() {
        assert!(!is_expired(1_000, 901, 100));
    }
}
