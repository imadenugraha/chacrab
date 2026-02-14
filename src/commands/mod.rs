pub mod add;
pub mod change_password;
pub mod delete;
pub mod export;
pub mod get;
pub mod import;
pub mod init;
pub mod list;
pub mod login;
pub mod logout;
pub mod update;

pub(crate) const VERIFICATION_SENTINEL: &str = "CHACRAB_VALID_SESSION";

pub(crate) fn verify_sentinel_constant_time(candidate: &str) -> bool {
	use subtle::ConstantTimeEq;

	let expected = VERIFICATION_SENTINEL.as_bytes();
	let candidate_bytes = candidate.as_bytes();

	let mut normalized = [0u8; VERIFICATION_SENTINEL.len()];
	let copy_len = candidate_bytes.len().min(normalized.len());
	normalized[..copy_len].copy_from_slice(&candidate_bytes[..copy_len]);

	let bytes_match = normalized.ct_eq(expected);
	let len_match = (candidate_bytes.len() as u64).ct_eq(&(expected.len() as u64));

	(bytes_match & len_match).into()
}

pub use add::add_credential;
pub use change_password::change_password;
pub use delete::delete_credential;
pub use export::export_credentials;
pub use get::get_credential;
pub use import::import_credentials;
pub use init::init_vault;
pub use list::list_credentials;
pub use login::login;
pub use logout::logout;
pub use update::update_credential_cmd;
