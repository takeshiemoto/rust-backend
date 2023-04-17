use validator::ValidationError;

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8
        || password.contains(char::is_whitespace)
        || !password.chars().any(char::is_numeric)
        || !password.chars().any(char::is_uppercase)
    {
        return Err(ValidationError::new("Invalid password"));
    }

    Ok(())
}
