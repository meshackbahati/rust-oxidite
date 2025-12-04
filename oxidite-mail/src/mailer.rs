use crate::{Result, transport::Transport, Message};
use async_trait::async_trait;

/// Mailer - Nodemailer-style API
pub struct Mailer<T: Transport> {
    transport: T,
}

impl<T: Transport> Mailer<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Send an email
    pub async fn send_mail(&self, message: Message) -> Result<()> {
        self.transport.send(message).await
    }

    /// Verify transport connection
    pub async fn verify(&self) -> Result<()> {
        self.transport.verify().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::SmtpTransport;

    #[tokio::test]
    #[ignore] // Requires SMTP server
    async fn test_send_email() {
        let transport = SmtpTransport::new("localhost", 1025).unwrap();
        let mailer = Mailer::new(transport);

        let message = Message::new()
            .from("sender@example.com")
            .to("recipient@example.com")
            .subject("Test Email")
            .text("Hello, World!");

        mailer.send_mail(message).await.unwrap();
    }
}
