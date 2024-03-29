use crate::{app_state::AppState, http_diff::types::AppError};
use ratatui::{prelude::*, widgets::*};

use super::{background::render_background, utils::centered_rect};

pub fn render_exception(frame: &mut Frame, app: &mut AppState) {
    match &app.critical_exception {
        Some(app_error) => {
            let mut text = vec![
                Line::from("** EXCEPTION **".bold().fg(app.theme.white)),
                Line::from("press `q` to exit".italic().fg(app.theme.white)),
                Line::from(""),
            ];

            let error = app_error.to_string();

            let lines: Vec<&str> = error.split("\n").collect();

            for line in lines {
                text.append(&mut vec![Line::from(line.fg(app.theme.white))]);
            }

            match app_error {
                AppError::FileNotFound(_) => {
                    text.append(&mut vec![
                        Line::from(""),
                        Line::from(
                        "Press `g` to generate a default configuration file"
                            .italic()
                            .fg(app.theme.white),
                    )]);
                }
                _ => {}
            }

            let popup_area = centered_rect(frame.size(), 60, 60);

            frame.render_widget(Clear, popup_area);

            render_background(frame, popup_area, &app.theme);

            let paragraph_block = Block::default().borders(Borders::ALL);
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(app.theme.gray))
                .block(paragraph_block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, popup_area);
        }
        _ => {}
    }
}
