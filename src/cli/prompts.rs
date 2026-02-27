use std::io::{self, Write};

use dialoguer::{Confirm, Editor, Input, Select, theme::ColorfulTheme};
use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroize;

use crate::core::errors::{ChacrabError, ChacrabResult};

pub fn confirmation_prompt(prompt: &str, default: bool) -> ChacrabResult<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default)
        .interact()
        .map_err(|_| ChacrabError::Config("unable to read confirmation".to_owned()))
}

pub fn secure_password_prompt(prompt: &str) -> ChacrabResult<SecretString> {
    print!("{}", prompt);
    io::stdout()
        .flush()
        .map_err(|_| ChacrabError::Config("unable to flush output".to_owned()))?;
    let password = rpassword::read_password()
        .map_err(|_| ChacrabError::Config("unable to read password".to_owned()))?;
    Ok(SecretString::new(password.into_boxed_str()))
}

pub fn optional_secure_password_prompt(prompt: &str) -> ChacrabResult<Option<SecretString>> {
    print!("{}", prompt);
    io::stdout()
        .flush()
        .map_err(|_| ChacrabError::Config("unable to flush output".to_owned()))?;
    let password = rpassword::read_password()
        .map_err(|_| ChacrabError::Config("unable to read password".to_owned()))?;
    if password.is_empty() {
        return Ok(None);
    }
    Ok(Some(SecretString::new(password.into_boxed_str())))
}

pub fn secure_password_with_confirmation(
    prompt: &str,
    confirm_prompt: &str,
) -> ChacrabResult<SecretString> {
    let first = secure_password_prompt(prompt)?;
    let second = secure_password_prompt(confirm_prompt)?;
    let mut second_plain = second.expose_secret().to_owned();

    if first.expose_secret() != second_plain {
        second_plain.zeroize();
        return Err(ChacrabError::Config(
            "password confirmation did not match".to_owned(),
        ));
    }

    second_plain.zeroize();

    Ok(first)
}

pub fn input(prompt: &str) -> ChacrabResult<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .map_err(|_| ChacrabError::Config("unable to read input".to_owned()))
}

pub fn optional_input(prompt: &str) -> ChacrabResult<Option<String>> {
    let value: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .map_err(|_| ChacrabError::Config("unable to read input".to_owned()))?;
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn multiline(prompt: &str) -> ChacrabResult<Option<String>> {
    Editor::new()
        .edit(prompt)
        .map_err(|_| ChacrabError::Config("unable to read multiline input".to_owned()))
}

pub fn select(prompt: &str, items: &[&str]) -> ChacrabResult<usize> {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .map_err(|_| ChacrabError::Config("unable to read selection".to_owned()))
}
