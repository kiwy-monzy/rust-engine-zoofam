//! Notifty (ntfy.sh) push notification client.
//!
//! Publishes real-time push notifications to user-specific topics.
//! Docs: https://docs.ntfy.sh/

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotiftyError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("server returned error: {status} {message}")]
    Server { status: u16, message: String },
}

#[derive(Clone)]
pub struct Notifty {
    server: String,
    topic_prefix: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
}


impl Notifty {
    pub fn new() -> Self {
        Self {
            server: std::env::var("NTFY_SERVER")
                .unwrap_or_else(|_| "https://ntfy.sh".to_string())
                .trim_end_matches('/')
                .to_string(),
            topic_prefix: std::env::var("NTFY_TOPIC_PREFIX").unwrap_or_else(|_| "gw".to_string()),
            username: std::env::var("NTFY_USER").ok().filter(|s| !s.is_empty()),
            password: std::env::var("NTFY_PASSWORD").ok().filter(|s| !s.is_empty()),
            client: reqwest::Client::new(),
        }
    }

    /// Returns the ntfy topic name for a given user id.
    pub fn topic_for(&self, user_id: &str) -> String {
        format!("{}-{}", self.topic_prefix, user_id.replace('-', ""))
    }

    /// Publish a notification to a user's topic.
    pub async fn publish(
        &self,
        user_id: &str,
        title: &str,
        message: &str,
        tags: Option<Vec<&str>>,
        priority: Option<u8>,
    ) -> Result<(), NotiftyError> {
        let topic = self.topic_for(user_id);
        let url = format!("{}/{}", self.server, topic);

        let mut headers = http::HeaderMap::new();
        headers.insert("Title", title.parse().unwrap());
        if let Some(ref t) = tags {
            headers.insert("Tags", t.join(",").parse().unwrap());
        }
        if let Some(p) = priority {
            let prio_str = match p {
                1..=2 => "min",
                3 => "default",
                4 => "high",
                5 => "urgent",
                _ => "default",
            };
            headers.insert("Priority", prio_str.parse().unwrap());
        }

        tracing::debug!("Notifty publish to {}: {} — {}", topic, title, message);

        let mut req = self
            .client
            .post(&url)
            .headers(headers)
            .body(message.as_bytes().to_vec());

        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            req = req.basic_auth(u, Some(p));
        }

        let resp = req.send().await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::error!("Notifty publish error {}: {}", status, body);
            return Err(NotiftyError::Server {
                status: status.as_u16(),
                message: body,
            });
        }

        tracing::info!("Notifty notification sent to topic {}", topic);
        Ok(())
    }

    /// Convenience: publish with default priority.
    pub async fn notify(
        &self,
        user_id: &str,
        title: &str,
        message: &str,
    ) -> Result<(), NotiftyError> {
        self.publish(user_id, title, message, None, None).await
    }
}

impl Default for Notifty {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic() {
        std::env::remove_var("NTFY_TOPIC_PREFIX");
        let n = Notifty::new();
        assert_eq!(n.topic_for("abc-1234-def"), "gw-abc1234def");
    }
}
