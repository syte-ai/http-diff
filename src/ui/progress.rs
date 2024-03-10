use ratatui::{prelude::*, widgets::*};

use std::cmp::max;

use crate::{app_state::AppState, http_diff::types::JobStatus};

pub fn render_progress_block(
    frame: &mut Frame,
    area: Rect,
    app: &mut AppState,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    let gauge = get_gauge(app);
    frame.render_widget(gauge, layout[1]);

    let sparkline = get_sparkle(app);

    frame.render_widget(sparkline, layout[0]);
}

pub fn get_sparkle<'a>(app: &'a mut AppState) -> Sparkline<'a> {
    let block = Block::default()
        .title("progress")
        .title_style(Style::default().fg(app.theme.white))
        .border_style(Style::default().fg(app.theme.white))
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);

    let mut content_length_copy = app.content_length_downloaded.clone();

    content_length_copy.sort_by(|a, b| b.cmp(a));

    let max_value_by_percent_index =
        (0.2 * content_length_copy.len() as f64) as usize;

    let max_value: u64 = max(
        *content_length_copy
            .get(max_value_by_percent_index)
            .unwrap_or_else(|| &100),
        1,
    );

    Sparkline::default()
        .block(block)
        .data(&app.content_length_downloaded)
        .max(max_value)
        .style(Style::default().fg(app.theme.gray))
}

pub fn get_gauge(app: &mut AppState) -> Gauge<'static> {
    let total_jobs_count = app.jobs.len();

    let in_progress_jobs_count = app
        .jobs
        .iter()
        .filter(|job| {
            job.status == JobStatus::Pending
                || job.status == JobStatus::Running
        })
        .count();

    let label = format!(
        "{}/{} jobs are finished",
        total_jobs_count - in_progress_jobs_count,
        total_jobs_count
    );

    let percentage: f64 = 100.0
        - (in_progress_jobs_count as f64 / total_jobs_count as f64) * 100.0;

    let block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(app.theme.white))
        .padding(Padding::new(0, 0, 1, 0));

    let foreground_color = if app.has_failed_jobs() {
        app.theme.warning
    } else {
        app.theme.success
    };

    Gauge::default()
        .block(block)
        .gauge_style(
            Style::new()
                .bg(app.theme.background)
                .fg(foreground_color)
                .italic(),
        )
        .percent(percentage as u16)
        .label(label)
}
