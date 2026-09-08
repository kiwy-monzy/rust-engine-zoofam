use ammonia::is_html;

use crate::Error;

pub const MIN_PASSWORD_LEN: usize = 8;
pub const MAX_PASSWORD_LEN: usize = 128;
pub const MAX_EMAIL_LEN: usize = 320;
pub const MAX_DISPLAY_NAME_LEN: usize = 120;
pub const MAX_AVATAR_URL_LEN: usize = 500;

const HTML_REJECTED: &str = "HTML/XSS content is not allowed";

pub fn email(value: &str) -> Result<String, Error> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.contains('@') {
        return Err(Error::Invalid("that is not an email address".into()));
    }
    if trimmed.len() > MAX_EMAIL_LEN {
        return Err(Error::Invalid(format!(
            "the email must be at most {MAX_EMAIL_LEN} characters"
        )));
    }
    Ok(trimmed.to_string())
}

pub fn password(value: &str) -> Result<(), Error> {
    let count = value.chars().count();
    if count < MIN_PASSWORD_LEN {
        return Err(Error::Invalid(format!(
            "the password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    if count > MAX_PASSWORD_LEN {
        return Err(Error::Invalid(format!(
            "the password must be at most {MAX_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}

pub fn display_name(value: &str) -> Result<(), Error> {
    plain_text(value, MAX_DISPLAY_NAME_LEN, "display name")
}

pub fn avatar_url(value: &str) -> Result<(), Error> {
    plain_text(value, MAX_AVATAR_URL_LEN, "avatar url")
}

fn plain_text(value: &str, max: usize, label: &str) -> Result<(), Error> {
    if value.chars().count() > max {
        return Err(Error::Invalid(format!(
            "the {label} must be at most {max} characters"
        )));
    }
    if is_html(value) {
        return Err(Error::Invalid(HTML_REJECTED.into()));
    }
    Ok(())
}

/// ISO-4217 currency code — three ASCII letters, all upper-case. We don't
/// maintain the full ISO list (that's 200+ entries); the regex is the
/// pragmatic check used by every payment system in practice.
pub fn currency(value: &str) -> Result<String, Error> {
    let v = value.trim();
    if v.len() != 3 || !v.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(Error::Invalid(
            "currency must be a 3-letter uppercase ISO-4217 code (e.g. TZS, USD, EUR)".into(),
        ));
    }
    Ok(v.to_string())
}
