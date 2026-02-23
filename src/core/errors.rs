use thiserror::Error;

pub type ChacrabResult<T> = Result<T, ChacrabError>;

#[derive(Debug, Error)]
pub enum ChacrabError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("no active session")]
    NoActiveSession,
    #[error("session expired")]
    SessionExpired,
    #[error("keyring unavailable")]
    KeyringUnavailable,
    #[error("keyring locked")]
    KeyringLocked,
    #[error("item not found")]
    NotFound,
    #[error("unsupported backend: {0}")]
    UnsupportedBackend(String),
    #[error("configuration error: {0}")]
    Config(String),
    #[error("crypto operation failed")]
    Crypto,
    #[error("serialization failed")]
    Serialization,
    #[error("storage operation failed")]
    Storage,
}

impl From<argon2::password_hash::Error> for ChacrabError {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::Crypto
    }
}

impl From<chacha20poly1305::aead::Error> for ChacrabError {
    fn from(_: chacha20poly1305::aead::Error) -> Self {
        Self::Crypto
    }
}

impl From<keyring::Error> for ChacrabError {
    fn from(err: keyring::Error) -> Self {
        match err {
            keyring::Error::NoEntry => Self::NoActiveSession,
            keyring::Error::NoStorageAccess(_) => Self::KeyringLocked,
            keyring::Error::PlatformFailure(_)
            | keyring::Error::BadEncoding(_)
            | keyring::Error::TooLong(_, _)
            | keyring::Error::Invalid(_, _)
            | keyring::Error::Ambiguous(_)
            | _ => Self::KeyringUnavailable,
        }
    }
}

impl From<serde_json::Error> for ChacrabError {
    fn from(_: serde_json::Error) -> Self {
        Self::Serialization
    }
}

impl From<sqlx::Error> for ChacrabError {
    fn from(_: sqlx::Error) -> Self {
        Self::Storage
    }
}

impl From<mongodb::error::Error> for ChacrabError {
    fn from(_: mongodb::error::Error) -> Self {
        Self::Storage
    }
}
