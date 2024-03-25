use std::cmp;

use crate::{
    app_state::AppState,
    http_diff::{job::JobDTO, request::Request},
};
use ratatui::{prelude::*, widgets::*};
use similar::ChangeTag;

use super::{
    background::render_background, theme::Theme, utils::centered_rect,
};

pub fn render_selected_job(frame: &mut Frame, app: &mut AppState) {
    match app.selected_job.as_mut() {
        Some(selected_job_state) => {
            let target_request = match selected_job_state
                .job
                .requests
                .get(selected_job_state.tab_index)
            {
                Some(request) => request,
                None => return,
            };

            selected_job_state.vertical_scroll_state = selected_job_state
                .vertical_scroll_state
                .content_length(target_request.diffs.len());

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

            let height = chunks[1].height as usize;

            let lines = map_request_to_lines(
                &app.theme,
                &target_request,
                height,
                selected_job_state.vertical_scroll,
            );

            let paragraph = Paragraph::new(lines)
                .style(Style::default().fg(app.theme.gray))
                .block(paragraph_block)
                .wrap(Wrap { trim: false });

            frame.render_widget(paragraph, chunks[1]);

            frame.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                chunks[1],
                &mut selected_job_state.vertical_scroll_state,
            );
        }
        _ => {}
    }
}

pub fn map_request_to_lines(
    theme: &Theme,
    request: &Request,
    window_height: usize,
    skip_lines: usize,
) -> Vec<Line<'static>> {
    let max_index_digits = (request.diffs.len() - 1).to_string().len();

    let mut content_window: Vec<Line<'static>> =
        Vec::with_capacity(window_height);

    let start_index = cmp::min(skip_lines, request.diffs.len() - 1);
    let end_index =
        cmp::min(request.diffs.len() - 1, skip_lines + window_height as usize);

    let content_slice = &request.diffs[start_index..=end_index];

    for (index, (tag, value)) in content_slice.iter().enumerate() {
        let index_formatted = format_index_prefix(
            tag,
            &(start_index + index),
            &max_index_digits,
        );

        let mapped_line = match tag {
            ChangeTag::Delete => Line::from(
                format!("{}{}", index_formatted, value)
                    .black()
                    .fg(theme.white)
                    .bg(theme.error),
            ),
            ChangeTag::Insert => Line::from(
                format!("{}{}", index_formatted, value)
                    .black()
                    .fg(theme.background)
                    .bg(theme.success),
            ),
            ChangeTag::Equal => {
                Line::from(format!("{}{}", index_formatted, value).gray())
            }
        };

        content_window.push(mapped_line);
    }

    content_window
}

fn format_index_prefix(
    tag: &ChangeTag,
    index: &usize,
    max_index: &usize,
) -> String {
    match tag {
        ChangeTag::Delete => {
            format!("{:width$}: - ", index, width = max_index)
        }
        ChangeTag::Insert => {
            format!("{:width$}: + ", index, width = max_index)
        }
        ChangeTag::Equal => format!("{:width$}:   ", index, width = max_index),
    }
}
pub struct SelectedJobState {
    pub tab_index: usize,
    pub job: JobDTO,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub scroll_boundary: ScrollBoundary,
    scroll_step: usize,
}

impl SelectedJobState {
    pub fn new(job: JobDTO, tab_index: usize) -> Self {
        let request = job.requests.get(tab_index).unwrap();

        let scroll_boundary = ScrollBoundary::new(&request.diffs);

        SelectedJobState {
            job,
            tab_index,
            scroll_boundary,
            vertical_scroll: 0,
            vertical_scroll_state: ScrollbarState::default(),
            scroll_step: 5,
        }
    }

    pub fn scroll_up(&mut self) {
        self.vertical_scroll = cmp::max(
            self.vertical_scroll.saturating_sub(self.scroll_step),
            self.scroll_boundary.y.0,
        );

        self.vertical_scroll_state =
            self.vertical_scroll_state.position(self.vertical_scroll);
    }

    pub fn scroll_down(&mut self) {
        self.vertical_scroll = cmp::min(
            self.vertical_scroll.saturating_add(self.scroll_step),
            self.scroll_boundary.y.1,
        );

        self.vertical_scroll_state =
            self.vertical_scroll_state.position(self.vertical_scroll);
    }
}

pub struct ScrollBoundary {
    pub x: (usize, usize),
    pub y: (usize, usize),
}

impl ScrollBoundary {
    pub fn new(diffs: &Vec<(ChangeTag, String)>) -> Self {
        let max_index_prefix = format_index_prefix(
            &ChangeTag::Insert,
            &diffs.len(),
            &diffs.len(),
        );

        let max_line_width =
            diffs.iter().map(|(_, line)| line.len()).max().unwrap_or_default()
                + max_index_prefix.len();

        ScrollBoundary { x: (0, max_line_width), y: (0, diffs.len()) }
    }
}
