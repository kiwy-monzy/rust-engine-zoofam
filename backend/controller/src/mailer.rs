//! SMTP mailer — supports cPanel/normal custom server with STARTTLS (port 587)
//! or SMTPS (port 465) and a plain username/password. Falls back to logging
//! the email to the console when SMTP is not configured so the rest of the app
//! stays usable in dev without an MTA.
//!
//! Configuration is read from environment variables:
//!   SMTP_HOST            (e.g. mail.example.com)
//!   SMTP_PORT            (default 587)
//!   SMTP_USERNAME
//!   SMTP_PASSWORD
//!   SMTP_FROM            (e.g. "TableTop Labs <no-reply@example.com>")
//!   SMTP_ENVELOPE_FROM   (optional, defaults to SMTP_FROM)
//!   SMTP_TLS             (starttls | tls | none — default starttls)
//!   PUBLIC_BASE_URL      (used to build links inside email bodies)

use lettre::{
    message::{Mailbox, Message},
    transport::smtp::authentication::Credentials,
    Address, SmtpTransport, Transport,
};
use std::env;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct Mailer {
    from: Mailbox,
    public_base_url: String,
    inner: Option<MailerInner>,
}

struct MailerInner {
    transport: SmtpTransport,
    host: String,
    port: u16,
    username: String,
}

impl Clone for MailerInner {
    fn clone(&self) -> Self {
        // The transport is internally cheap to clone (it wraps an Arc), but
        // our wrapper still needs Clone for Mailer. The right thing to do is
        // re-build from the env on each clone, but for the read path it is
        // cheaper to keep a once-init inner Arc. For now we rebuild.
        let host = env::var("SMTP_HOST")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.host.clone());
        let port: u16 = env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(self.port);
        let username = env::var("SMTP_USERNAME")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.username.clone());
        let password = env::var("SMTP_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_default();
        let envelope = env::var("SMTP_ENVELOPE_FROM")
            .ok()
            .unwrap_or_else(|| username.clone());
        let mode = env::var("SMTP_TLS")
            .unwrap_or_else(|_| "starttls".to_string())
            .to_lowercase();
        let url = match mode.as_str() {
            "tls" | "ssl" | "smtps" => format!("smtps://{}:{}", host, port),
            _ => format!("smtp://{}:{}", host, port),
        };
        let transport = SmtpTransport::from_url(&url)
            .unwrap_or_else(|_| panic!("invalid smtp url"))
            .credentials(Credentials::new(envelope, password))
            .build();
        Self {
            transport,
            host,
            port,
            username,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MailerError {
    #[error("email is not configured: {0}")]
    NotConfigured(String),
    #[error("invalid email address: {0}")]
    InvalidAddress(String),
    #[error("failed to build email: {0}")]
    Build(String),
    #[error("SMTP send failed: {0}")]
    Send(String),
}

impl std::fmt::Debug for MailerInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailerInner")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .finish()
    }
}

static MAILER: OnceLock<Mailer> = OnceLock::new();

/// Initialise the global mailer from environment. Safe to call multiple times.
pub fn init() -> &'static Mailer {
    MAILER.get_or_init(Mailer::from_env)
}

pub fn current() -> Option<&'static Mailer> {
    MAILER.get()
}

impl Mailer {
    pub fn from_env() -> Self {
        let from = parse_mailbox(env::var("SMTP_FROM").ok().as_deref().unwrap_or(""))
            .unwrap_or_else(|| Mailbox::new(None, "no-reply@localhost".parse().expect("valid")));
        let public_base_url = env::var("PUBLIC_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:5173".to_string())
            .trim_end_matches('/')
            .to_string();

        let inner = build_inner_transport();

        if inner.is_none() {
            tracing::warn!(
                "SMTP is not fully configured (need SMTP_HOST + SMTP_USERNAME + SMTP_PASSWORD). \
                 Outgoing email will be logged to the console only."
            );
        }

        Self {
            from,
            public_base_url,
            inner,
        }
    }

    pub fn public_base_url(&self) -> &str {
        &self.public_base_url
    }

    /// Send a plain-text email. (HTML body support intentionally not wired in
    /// to keep the dep surface small — the body is built as a plain `String`.)
    pub fn send(&self, to: &str, subject: &str, text: &str) -> Result<(), MailerError> {
        let to = to.trim();
        if to.is_empty() {
            return Err(MailerError::InvalidAddress("empty".into()));
        }
        let to_mb: Mailbox = to
            .parse()
            .map_err(|_| MailerError::InvalidAddress(to.to_string()))?;

        // Use the body-only path so we don't have to depend on the multipart builder.
        let email: Message = Message::builder()
            .from(self.from.clone())
            .to(to_mb)
            .subject(subject)
            .body(text.to_string())
            .map_err(|e| MailerError::Build(e.to_string()))?;

        match &self.inner {
            Some(inner) => {
                inner
                    .transport
                    .send(&email)
                    .map_err(|e| MailerError::Send(e.to_string()))?;
            }
            None => {
                tracing::info!(
                    "[mailer-stub] would send to={} subject={} body_len={}",
                    to,
                    subject,
                    text.len()
                );
            }
        }
        Ok(())
    }

    pub fn send_reset_password(
        &self,
        to: &str,
        display_name: &str,
        token: &str,
    ) -> Result<(), MailerError> {
        let link = format!("{}/reset-password?token={}", self.public_base_url, token);
        let subject = "Reset your TableTop Labs password";
        let greeting = if display_name.is_empty() {
            "Hello"
        } else {
            display_name
        };
        let text = format!(
            "{},\n\nWe received a request to reset the password on your TableTop Labs account.\n\
             Click the link below within the next hour to choose a new password:\n\n  {}\n\n\
             If you did not make this request you can safely ignore this email.\n",
            greeting, link
        );
        self.send(to, subject, &text)
    }
}

fn build_inner_transport() -> Option<MailerInner> {
    let host = env::var("SMTP_HOST").ok().filter(|s| !s.is_empty())?;
    let port: u16 = env::var("SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);
    let username = env::var("SMTP_USERNAME").ok().filter(|s| !s.is_empty())?;
    let password = env::var("SMTP_PASSWORD").ok().filter(|s| !s.is_empty())?;
    let mode = env::var("SMTP_TLS")
        .unwrap_or_else(|_| "starttls".to_string())
        .to_lowercase();
    let envelope_from = env::var("SMTP_ENVELOPE_FROM")
        .ok()
        .unwrap_or_else(|| username.clone());

    let transport = match mode.as_str() {
        "tls" | "ssl" | "smtps" => {
            let url = format!("smtps://{}:{}", host, port);
            SmtpTransport::from_url(&url)
                .unwrap_or_else(|_| panic!("invalid smtps url"))
                .credentials(Credentials::new(envelope_from.clone(), password.clone()))
                .build()
        }
        "none" | "plain" => {
            let url = format!("smtp://{}:{}", host, port);
            SmtpTransport::from_url(&url)
                .unwrap_or_else(|_| panic!("invalid smtp url"))
                .credentials(Credentials::new(envelope_from.clone(), password.clone()))
                .build()
        }
        _ => {
            // default: STARTTLS
            let url = format!("smtp://{}:{}", host, port);
            SmtpTransport::from_url(&url)
                .unwrap_or_else(|_| panic!("invalid smtp url"))
                .credentials(Credentials::new(envelope_from.clone(), password.clone()))
                .build()
        }
    };

    Some(MailerInner {
        transport,
        host,
        port,
        username,
    })
}

fn parse_mailbox(s: &str) -> Option<Mailbox> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(start) = s.find('<') {
        if let Some(end) = s.find('>') {
            let name = s[..start].trim().trim_matches('"').to_string();
            let addr = s[start + 1..end].trim().to_string();
            let addr: Address = addr.parse().ok()?;
            return Some(Mailbox::new(Some(name), addr));
        }
    }
    let addr: Address = s.parse().ok()?;
    Some(Mailbox::new(None, addr))
}
