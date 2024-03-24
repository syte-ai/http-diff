use crate::{app_state::AppState, http_diff::request::Request};
use ratatui::{prelude::*, widgets::*};
use similar::ChangeTag;

use super::{
    background::render_background, theme::Theme, utils::centered_rect,
};

pub fn render_selected_job(frame: &mut Frame, app: &mut AppState) {
    match &app.selected_job {
        Some(selected_job_state) => {
            app.vertical_scroll_state = app
                .vertical_scroll_state
                .content_length(selected_job_state.current_tabs_content.len());

            let popup_area = centered_rect(frame.size(), 95, 95);

            frame.render_widget(Clear, popup_area);

            render_background(frame, popup_area, &app.theme);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Min(10)])
                .split(popup_area);

            let titles = selected_job_state
                .job
                .requests
                .iter()
                .map(|request| {
                    let title = request.uri.domain().unwrap_or("");

                    if request.has_diffs {
                        return Line::from(
                            title
                                .black()
                                .fg(app.theme.white)
                                .bg(app.theme.error),
                        );
                    }

                    return Line::from(
                        title
                            .black()
                            .fg(app.theme.white)
                            .bg(app.theme.success),
                    );
                })
                .collect();

            let tabs_block = Block::default()
                .title(format!(
                    "Endpoint: {}",
                    selected_job_state.job.job_name
                ))
                .borders(Borders::RIGHT | Borders::LEFT | Borders::TOP)
                .title_style(Style::default().fg(app.theme.gray))
                .border_style(Style::default().fg(app.theme.gray));

            let tabs = Tabs::new(titles)
                .block(tabs_block)
                .select(selected_job_state.tab_index)
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_widget(tabs, chunks[0]);

            let paragraph_block = Block::default()
                .borders(Borders::RIGHT | Borders::LEFT | Borders::BOTTOM);

            let paragraph = Paragraph::new(
                selected_job_state.current_tabs_content.clone(),
            )
            .style(Style::default().fg(app.theme.gray))
            .block(paragraph_block)
            .wrap(Wrap { trim: false })
            .scroll((app.vertical_scroll as u16, 0));

            frame.render_widget(paragraph, chunks[1]);

            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                chunks[1],
                &mut app.vertical_scroll_state,
            );
        }
        _ => {}
    }
}

pub fn map_request_to_lines(
    theme: &Theme,
    request: &Request,
) -> Vec<Line<'static>> {
    let max_index_digits = (request.diffs.len() - 1).to_string().len();

    let lines: Vec<Line<'_>> = request
        .diffs
        .iter()
        .enumerate()
        .map(|(index, (change_tag, value))| {
            let index_formatted =
                format!("{:width$}", index, width = max_index_digits);

            match change_tag {
                ChangeTag::Delete => Line::from(
                    format!("{}: - {}", index_formatted, value)
                        .black()
                        .fg(theme.white)
                        .bg(theme.error),
                ),
                ChangeTag::Insert => Line::from(
                    format!("{}: + {}", index_formatted, value)
                        .black()
                        .fg(theme.background)
                        .bg(theme.success),
                ),
                ChangeTag::Equal => Line::from(
                    format!("{}:  {}", index_formatted, value).gray(),
                ),
            }
        })
        .collect();

    lines
}
