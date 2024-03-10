use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: String,
    pub body: String,
    pub expire_duration: Option<Duration>,
    pub started_at: Instant,
    pub r#type: NotificationType,
}

impl Notification {
    pub fn new(
        id: &str,
        body: &str,
        expire_duration: Option<Duration>,
        r#type: NotificationType,
    ) -> Self {
        Self {
            id: id.to_owned(),
            expire_duration,
            body: body.to_owned(),
            started_at: Instant::now(),
            r#type,
        }
    }

    pub fn get_show_percentage_left(&self) -> Option<u64> {
        if let Some(expire) = self.expire_duration {
            let now = Instant::now();

            let left = now - self.started_at;

            return Some(100 - (left.as_secs() * 100) / expire.as_secs());
        };

        None
    }
}
