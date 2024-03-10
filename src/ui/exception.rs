use crate::app_state::AppState;
use ratatui::{prelude::*, widgets::*};

use super::{background::render_background, utils::centered_rect};

pub fn render_exception(frame: &mut Frame, app: &mut AppState) {
    match &app.critical_exception {
        Some(error) => {
            let mut text = vec![
                Line::from("** EXCEPTION **".bold().fg(app.theme.white)),
                Line::from("press `q` to exit".italic().fg(app.theme.white)),
                Line::from(""),
            ];

            let error = error.to_string();

            let lines: Vec<&str> = error.split("\n").collect();

            for line in lines {
                text.append(&mut vec![Line::from(line.fg(app.theme.white))]);
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
