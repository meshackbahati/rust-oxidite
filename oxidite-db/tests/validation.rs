use oxidite_db::{Model, sqlx};

#[derive(Model, sqlx::FromRow, Clone)]
struct UserWithValidation {
    id: i64,
    username: String,
    #[validate(email)]
    email: String,
}

#[test]
fn test_email_validation_valid() {
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "test@example.com".to_string(),
    };
    
    assert!(user.validate().is_ok());
}

#[test]
fn test_email_validation_invalid() {
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "invalid-email".to_string(),
    };
    
    let result = user.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid email format"));
}

#[test]
fn test_email_validation_missing_at() {
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "testexample.com".to_string(),
    };
    
    assert!(user.validate().is_err());
}

#[test]
fn test_email_validation_missing_domain() {
    let user = UserWithValidation {
        id: 1,
        username: "test".to_string(),
        email: "test@".to_string(),
    };
    
    assert!(user.validate().is_err());
}
