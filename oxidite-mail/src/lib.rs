// Nodemailer-style email library for Rust

/// Mailer module for sending emails
pub mod mailer;
/// Email message builder module
pub mod message;
/// SMTP transport module
pub mod transport;
/// Email attachment module
pub mod attachment;

/// Re-export of [`Mailer`]
pub use mailer::Mailer;
/// Re-export of [`Message`]
pub use message::Message;
/// Re-export of [`SmtpTransport`] and [`SmtpConfig`]
pub use transport::{SmtpTransport, SmtpConfig};
/// Re-export of [`Attachment`]
pub use attachment::Attachment;

/// Email errors
#[derive(Debug, thiserror::Error)]
pub enum MailError {
    /// SMTP protocol error
    #[error("SMTP error: {0}")]
    Smtp(String),
    
    /// Invalid email address
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Attachment error
    #[error("Attachment error: {0}")]
    Attachment(String),
    
    /// Underlying lettre transport error
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    
    /// Address parsing error
    #[error("Address error: {0}")]
    Address(#[from] lettre::address::AddressError),
    
    /// Email building error
    #[error("Email building error: {0}")]
    EmailBuilder(#[from] lettre::error::Error),
}

/// Mail result type alias
pub type Result<T> = std::result::Result<T, MailError>;
