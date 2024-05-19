use crate::app_state::AppState;
use crate::http_diff::job::JobDTO;
use crate::http_diff::types::JobStatus;
use ratatui::{prelude::*, widgets::*};

use super::theme::Theme;

pub fn get_headers_from_domains(domains: &Vec<String>) -> Vec<&str> {
    let mut headers: Vec<&str> = vec!["Endpoint"];

    for domain in domains {
        headers.push(domain.as_str())
    }

    headers
}

pub fn render_jobs_table(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(app.theme.gray);

    let headers = get_headers_from_domains(&app.domains);

    let header_cells = headers.iter().map(|h| {
        Cell::from(*h).style(
            Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD),
        )
    });

    let header =
        Row::new(header_cells).style(normal_style).height(1).bottom_margin(1);

    let rows = app.jobs.iter().map(|job| {
        let cells = get_cells_from_job(job, &app.theme);

        Row::new(cells).height(1).bottom_margin(1)
    });

    let mut widths = vec![Constraint::Percentage(40)];

    // Calculate the width for each remaining column
    if headers.len() > 1 {
        let remaining_width = 60; // 100% - 40% for the first column
        let column_width = remaining_width / (headers.len() - 1) as u16;

        for _ in 0..(headers.len() - 1) {
            widths.push(Constraint::Percentage(column_width));
        }
    }

    let table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("jobs")
                .title_style(Style::default().fg(app.theme.white))
                .border_style(Style::default().fg(app.theme.white)),
        )
        .highlight_style(selected_style)
        .highlight_symbol("> ")
        .column_spacing(3)
        .widths(&widths);

    frame.render_stateful_widget(table, area, &mut app.state);
}

fn get_cells_from_job<'a>(job: &'a JobDTO, theme: &'a Theme) -> Vec<Cell<'a>> {
    let mut cells = vec![Cell::from(job.job_name.as_str().fg(theme.white))];

    for request in &job.requests {
        let cell = match request.status {
            JobStatus::Finished => Cell::from(request.get_status_text())
                .style(
                    Style::default()
                        .bg(theme.success)
                        .fg(theme.white)
                        .add_modifier(Modifier::BOLD),
                ),
            JobStatus::Failed => Cell::from(request.get_status_text())
                .style(Style::default().bg(theme.error))
                .fg(theme.white)
                .add_modifier(Modifier::BOLD),
            _ => Cell::from(request.get_status_text())
                .style(Style::default().bg(theme.white))
                .fg(theme.background),
        };

        cells.push(cell);
    }

    cells
}
