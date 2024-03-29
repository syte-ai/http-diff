use ratatui::{prelude::*, widgets::*};

use crate::app_state::AppState;

fn notification_rect(r: Rect) -> Rect {
    let popup_vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(85),
            Constraint::Max(7),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(3),
            Constraint::Max(70),
            Constraint::Min(9),
        ])
        .split(popup_vertical_layout[1])[1]
}

pub fn render_notification(frame: &mut Frame, app: &mut AppState) {
    if let Some(notification) = &app.notification {
        let popup_area = notification_rect(frame.size());

        frame.render_widget(Clear, popup_area);

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(85),
                Constraint::Percentage(15),
            ])
            .split(popup_area);

        let block = Block::default().bg(match notification.r#type {
            NotificationType::Error => app.theme.error,
            NotificationType::Warning => app.theme.warning,
            NotificationType::Success => app.theme.success,
        });

        frame.render_widget(block, popup_area);

        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .title(match notification.r#type {
                NotificationType::Error => "Error",
                NotificationType::Warning => "Warning",
                NotificationType::Success => "Success",
            })
            .style(match notification.r#type {
                NotificationType::Error => {
                    Style::default().fg(app.theme.black)
                }
                NotificationType::Warning => {
                    Style::default().fg(app.theme.black)
                }
                NotificationType::Success => {
                    Style::default().fg(app.theme.white)
                }
            });

        let body: Vec<Line<'_>> = notification
            .body
            .as_str()
            .lines()
            .map(|line| Line::from(line.fg(app.theme.black)))
            .collect();

        let paragraph = Paragraph::new(body)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .block(block);

        if let Some(percentage) = notification.get_show_percentage_left() {
            let gauge = Gauge::default()
                .gauge_style(Style::new().gray())
                .percent(percentage as u16);

            frame.render_widget(gauge, popup_layout[1]);
        } else {
            let block = Block::default().padding(Padding::horizontal(1));

            let paragraph = Paragraph::new("Press `Esc` to dismiss".gray())
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Right)
                .block(block);

            frame.render_widget(paragraph, popup_layout[1]);
        }

        frame.render_widget(paragraph, popup_layout[0]);
    }
}

use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationId {
    NoFailedJobs,
    SavedJob,
    SavedJobs,
    FailedToSaveJobs,
    PendingJobInfoError,
    JobProgressChange,
    GenerateDefaultConfig,
    GenerateDefaultConfigFailed,
    AllRequestsFinishedWithoutFails,
    ReloadingConfiguration,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Notification {
    pub id: NotificationId,
    pub body: String,
    pub expire_duration: Option<Duration>,
    pub started_at: Instant,
    pub r#type: NotificationType,
}

impl Notification {
    pub fn new(
        id: NotificationId,
        body: &str,
        expire_duration: Option<Duration>,
        r#type: NotificationType,
    ) -> Self {
        Self {
            id,
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
