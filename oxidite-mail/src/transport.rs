use crate::{Message, Result, MailError};
use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
};
use lettre::transport::smtp::authentication::Credentials;

/// Transport trait for sending emails
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, message: Message) -> Result<()>;
    async fn verify(&self) -> Result<()>;
}

/// SMTP transport configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}

impl SmtpConfig  {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            username: None,
            password: None,
            use_tls: true,
        }
    }

    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }
}

/// SMTP transport
pub struct SmtpTransport {
    config: SmtpConfig,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpTransport {
    pub fn new(host: impl Into<String>, port: u16) -> Result<Self> {
        let config = SmtpConfig::new(host, port);
        let transport = Self::build_transport(&config)?;
        
        Ok(Self { config, transport })
    }

    pub fn from_config(config: SmtpConfig) -> Result<Self> {
        let transport = Self::build_transport(&config)?;
        Ok(Self { config, transport })
    }

    fn build_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let mut builder = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
                .map_err(|e| MailError::Smtp(e.to_string()))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        };

        builder = builder.port(config.port);

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
        }

        Ok(builder.build())
    }

    fn build_email(&self, message: Message) -> Result<lettre::Message> {
        message.validate()?;

        let from: Mailbox = message.from.as_ref().unwrap().parse()?;
        let to: Vec<Mailbox> = message.to.iter()
            .map(|addr| addr.parse())
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let mut email_builder = lettre::Message::builder()
            .from(from)
            .subject(message.subject.as_ref().unwrap());

        for recipient in to {
            email_builder = email_builder.to(recipient);
        }

        for cc in &message.cc {
            email_builder = email_builder.cc(cc.parse()?);
        }

        for bcc in &message.bcc {
            email_builder = email_builder.bcc(bcc.parse()?);
        }

        if let Some(reply_to) = &message.reply_to {
            email_builder = email_builder.reply_to(reply_to.parse()?);
        }

        // Build body
        let mut body = if let (Some(text), Some(html)) = (&message.text, &message.html) {
            MultiPart::alternative_plain_html(text.clone(), html.clone())
        } else if let Some(html) = &message.html {
            MultiPart::alternative()
                .singlepart(SinglePart::html(html.clone()))
        } else if let Some(text) = &message.text {
            MultiPart::alternative()
                .singlepart(SinglePart::plain(text.clone()))
        } else {
            return Err(MailError::MissingField("text or html".to_string()));
        };

        // Add attachments
        if !message.attachments.is_empty() {
            let mut multipart = MultiPart::mixed().multipart(body);

            for attachment in &message.attachments {
                let content_type = if let Some(ct) = &attachment.content_type {
                    ContentType::parse(ct).unwrap_or(ContentType::TEXT_PLAIN)
                } else {
                    ContentType::TEXT_PLAIN
                };

                let part = SinglePart::builder()
                    .header(content_type)
                    .body(attachment.content.clone());

                multipart = multipart.singlepart(part);
            }

            body = multipart;
        }

        let email = email_builder.multipart(body)?;
        Ok(email)
    }
}

#[async_trait]
impl Transport for SmtpTransport {
    async fn send(&self, message: Message) -> Result<()> {
        let email = self.build_email(message)?;
        self.transport.send(email).await?;
        Ok(())
    }

    async fn verify(&self) -> Result<()> {
        self.transport.test_connection().await
            .map_err(|e| MailError::Smtp(e.to_string()))?;
        Ok(())
    }
}
