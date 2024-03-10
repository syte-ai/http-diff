use crate::app_state::AppState;
use ratatui::{prelude::*, widgets::*};
use similar::ChangeTag;

use super::{background::render_background, utils::centered_rect};

pub fn render_selected_job(frame: &mut Frame, app: &mut AppState) {
    match &app.selected_job {
        Some(selected_job_state) => {
            let requests_with_diffs =
                selected_job_state.job.get_requests_with_diffs();

            let current_request = if let Some(request) =
                requests_with_diffs.get(selected_job_state.tab_index)
            {
                request
            } else {
                return;
            };

            let max_index_digits =
                (current_request.diffs.len() - 1).to_string().len();

            let text: Vec<Line<'_>> = current_request
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
                                .fg(app.theme.white)
                                .bg(app.theme.error),
                        ),
                        ChangeTag::Insert => Line::from(
                            format!("{}: + {}", index_formatted, value)
                                .black()
                                .fg(app.theme.background)
                                .bg(app.theme.success),
                        ),
                        ChangeTag::Equal => Line::from(
                            format!("{}:  {}", index_formatted, value).gray(),
                        ),
                    }
                })
                .collect();

            app.vertical_scroll_state =
                app.vertical_scroll_state.content_length(text.len());

            let popup_area = centered_rect(frame.size(), 95, 95);

            frame.render_widget(Clear, popup_area);

            render_background(frame, popup_area, &app.theme);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Min(10)])
                .split(popup_area);

            let titles = requests_with_diffs
                .iter()
                .map(|request| {
                    let title = request.uri.domain().unwrap_or("");

                    Line::from(title)
                })
                .collect();

            let tabs_block = Block::default()
                .title(format!(
                    "Diffs for endpoint: {}",
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
                        .fg(app.theme.error)
                        .bg(app.theme.gray)
                        .add_modifier(Modifier::BOLD),
                );

            frame.render_widget(tabs, chunks[0]);

            let paragraph_block = Block::default()
                .borders(Borders::RIGHT | Borders::LEFT | Borders::BOTTOM);

            let paragraph = Paragraph::new(text)
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
