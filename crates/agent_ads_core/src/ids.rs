use crate::error::{MetaAdsError, Result};

pub fn normalize_account_id(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(MetaAdsError::InvalidArgument(
            "account id cannot be empty".to_string(),
        ));
    }

    if let Some(stripped) = trimmed.strip_prefix("act_") {
        return digits_only(stripped).map(|_| trimmed.to_string());
    }

    digits_only(trimmed)?;
    Ok(format!("act_{trimmed}"))
}

fn digits_only(value: &str) -> Result<()> {
    if value.chars().all(|ch| ch.is_ascii_digit()) {
        Ok(())
    } else {
        Err(MetaAdsError::InvalidArgument(format!(
            "expected a numeric ad account id, got `{value}`"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_account_id;

    #[test]
    fn adds_prefix_when_missing() {
        assert_eq!(normalize_account_id("123").unwrap(), "act_123");
    }

    #[test]
    fn keeps_prefixed_value() {
        assert_eq!(normalize_account_id("act_123").unwrap(), "act_123");
    }

    #[test]
    fn rejects_invalid_value() {
        assert!(normalize_account_id("acct_123").is_err());
    }
}
