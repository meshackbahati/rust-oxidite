use crate::{Attachment, Result, MailError};

/// Email message builder (Nodemailer-style)
#[derive(Debug, Clone)]
pub struct Message {
    pub(crate) from: Option<String>,
    pub(crate) to: Vec<String>,
    pub(crate) cc: Vec<String>,
    pub(crate) bcc: Vec<String>,
    pub(crate) reply_to: Option<String>,
    pub(crate) subject: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) html: Option<String>,
    pub(crate) attachments: Vec<Attachment>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            from: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: None,
            text: None,
            html: None,
            attachments: Vec::new(),
        }
    }

    /// Set sender address
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Add recipient
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to.push(to.into());
        self
    }

    /// Add CC recipient
    pub fn cc(mut self, cc: impl Into<String>) -> Self {
        self.cc.push(cc.into());
        self
    }

    /// Add BCC recipient
    pub fn bcc(mut self, bcc: impl Into<String>) -> Self {
        self.bcc.push(bcc.into());
        self
    }

    /// Set reply-to address
    pub fn reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Set subject
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set plain text body
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set HTML body
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Add attachment
    pub fn attach(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    /// Validate message
    pub(crate) fn validate(&self) -> Result<()> {
        if self.from.is_none() {
            return Err(MailError::MissingField("from".to_string()));
        }
        if self.to.is_empty() {
            return Err(MailError::MissingField("to".to_string()));
        }
        if self.subject.is_none() {
            return Err(MailError::MissingField("subject".to_string()));
        }
        if self.text.is_none() && self.html.is_none() {
            return Err(MailError::MissingField("text or html".to_string()));
        }
        Ok(())
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}
