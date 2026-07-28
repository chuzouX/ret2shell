use r2s_database::oauth_provider;

use crate::{traits::ResponseError, utility::string::account_str};

fn char_len(value: &str) -> usize {
  value.chars().count()
}

fn validate_required(value: &str, field: &str) -> Result<(), ResponseError> {
  if value.trim().is_empty() {
    return Err(ResponseError::BadRequest(format!("{field} is required")));
  }
  Ok(())
}

pub fn validate_account(account: &str) -> Result<(), ResponseError> {
  let len = char_len(account);
  if len < 4 {
    return Err(ResponseError::BadRequest(
      "account must be at least 4 characters".to_owned(),
    ));
  }
  if len > 32 {
    return Err(ResponseError::BadRequest(
      "account must be at most 32 characters".to_owned(),
    ));
  }
  if account_str(account) != account {
    return Err(ResponseError::BadRequest(
      "account contains invalid characters".to_owned(),
    ));
  }
  Ok(())
}

pub fn validate_nickname(nickname: &str) -> Result<(), ResponseError> {
  let len = char_len(nickname);
  if len < 2 {
    return Err(ResponseError::BadRequest(
      "nickname must be at least 2 characters".to_owned(),
    ));
  }
  if len > 32 {
    return Err(ResponseError::BadRequest(
      "nickname must be at most 32 characters".to_owned(),
    ));
  }
  Ok(())
}

pub fn validate_email(email: &str) -> Result<(), ResponseError> {
  let Some((local, domain)) = email.split_once('@') else {
    return Err(ResponseError::BadRequest("invalid email".to_owned()));
  };
  if local.is_empty()
    || domain.is_empty()
    || domain.contains('@')
    || email.chars().any(char::is_whitespace)
    || !domain
      .split('.')
      .all(|label| !label.is_empty() && !label.starts_with('-') && !label.ends_with('-'))
    || !domain.contains('.')
  {
    return Err(ResponseError::BadRequest("invalid email".to_owned()));
  }
  Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ResponseError> {
  let len = char_len(password);
  if !(8..=40).contains(&len)
    || !password.chars().any(|c| c.is_ascii_lowercase())
    || !password.chars().any(|c| c.is_ascii_uppercase())
    || !password.chars().any(|c| c.is_ascii_digit())
  {
    return Err(ResponseError::BadRequest("password is too weak".to_owned()));
  }
  Ok(())
}

pub fn validate_register_request(
  account: &str, nickname: &str, email: &str, password: &str,
) -> Result<(), ResponseError> {
  validate_account(account)?;
  validate_nickname(nickname)?;
  validate_email(email)?;
  validate_password(password)?;
  Ok(())
}

pub fn validate_oauth_provider_model(
  provider: &oauth_provider::Model,
) -> Result<(), ResponseError> {
  validate_required(&provider.name, "oauth provider name")?;
  let provider_len = char_len(&provider.provider);
  if !(2..=32).contains(&provider_len)
    || !provider
      .provider
      .chars()
      .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
  {
    return Err(ResponseError::BadRequest(
      "oauth provider contains invalid characters".to_owned(),
    ));
  }
  validate_required(&provider.script, "oauth provider script")?;
  if let Some(portal) = &provider.portal
    && !portal.trim().is_empty()
    && (!portal.starts_with("https://") && !portal.starts_with("http://")
      || portal.chars().any(char::is_whitespace))
  {
    return Err(ResponseError::BadRequest(
      "oauth provider portal is invalid".to_owned(),
    ));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::{
    validate_account, validate_email, validate_nickname, validate_password,
    validate_register_request,
  };

  #[test]
  fn register_validation_accepts_frontend_valid_fields() {
    assert!(
      validate_register_request(
        "Valid_User_01",
        "测试用户",
        "user@example.com",
        "StrongPass1"
      )
      .is_ok()
    );
  }

  #[test]
  fn register_validation_rejects_invalid_accounts_after_filtering() {
    assert!(validate_account("abc").is_err());
    assert!(validate_account("a".repeat(33).as_str()).is_err());
    assert!(validate_account("bad-user").is_err());
    assert!(validate_account("bad user").is_err());
    assert!(validate_account("测试_user").is_err());
  }

  #[test]
  fn register_validation_rejects_invalid_nickname_email_and_password() {
    assert!(validate_nickname("a").is_err());
    assert!(validate_nickname("a".repeat(33).as_str()).is_err());
    assert!(validate_email("not-an-email").is_err());
    assert!(validate_email("user@example").is_err());
    assert!(validate_password("weakpass1").is_err());
    assert!(validate_password("WEAKPASS1").is_err());
    assert!(validate_password("WeakPass").is_err());
    assert!(validate_password("Aa1".repeat(14).as_str()).is_err());
  }
}
