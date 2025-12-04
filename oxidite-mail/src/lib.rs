// Nodemailer-style email library for Rust

pub mod mailer;
pub mod message;
pub mod transport;
pub mod attachment;

pub use mailer::Mailer;
pub use message::Message;
pub use transport::{SmtpTransport, SmtpConfig};
pub use attachment::Attachment;

/// Email errors
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    #[error("SMTP error: {0}")]
    Smtp(String),
    
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Attachment error: {0}")]
    Attachment(String),
    
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    
    #[error("Address error: {0}")]
    Address(#[from] lettre::address::AddressError),
    
    #[error("Email building error: {0}")]
    EmailBuilder(#[from] lettre::error::Error),
}

pub type Result<T> = std::result::Result<T, MailError>;
