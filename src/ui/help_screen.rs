use crate::app_state::{AppState, Screen};
use ratatui::{prelude::*, widgets::*};

use super::{background::render_background, utils::centered_rect};

pub fn render_help_popup(frame: &mut Frame, app: &mut AppState) {
    match app.should_show_help {
        true => {
            let mut text = vec![
                Line::from("- `q` to quit".italic().fg(app.theme.white)),
                Line::from("- `t` to change the theme".italic().fg(app.theme.white)),
                Line::from(
                    format!(
                        "- `s` to save selected row to output directory: {}",
                        app.output_directory.to_str().unwrap_or("")
                    )
                    .italic()
                    .fg(app.theme.white),
                ),
                Line::from(
                    format!(
                        "- `Shift + s` to save all failed rows to output directory: {}",
                        app.output_directory.to_str().unwrap_or("")
                    )
                    .italic()
                    .fg(app.theme.white),
                ),
                Line::from("- `r` to restart selected job".italic().fg(app.theme.white)),
                Line::from(
                    "- `Shift + r` to restart all jobs"
                        .italic()
                        .fg(app.theme.white),
                ),
                Line::from("- `Esc` to close windows".italic().fg(app.theme.white)),
                Line::from(
                    "- `⬆` and `⬇` keys to navigate"
                        .italic()
                        .fg(app.theme.white),
                ),
            ];

            match app.current_screen {
                Screen::JobInfo => {
                    text.push(Line::from(
                        "- `Shift + ⬇` to go to the next diff"
                            .italic()
                            .fg(app.theme.white),
                    ));

                    text.push(Line::from(
                        "- `Shift + ⬆` to go to the previous diff"
                            .italic()
                            .fg(app.theme.white),
                    ));

                    text.push(Line::from(
                        "- `Shift + →` to go to the next failed request"
                            .italic()
                            .fg(app.theme.white),
                    ));

                    text.push(Line::from(
                        "- `Shift + ←` to go to the previous failed request"
                            .italic()
                            .fg(app.theme.white),
                    ));

                    text.push(Line::from(
                        "- `Tab` to switch to the next domain if available"
                            .italic()
                            .fg(app.theme.white),
                    ));
                }
                Screen::Home => text.push(Line::from(
                    "- `Enter` to show the response of the selected job"
                        .italic()
                        .fg(app.theme.white),
                )),
                _ => {}
            }

            let popup_area = centered_rect(frame.size(), 40, 40);

            let block = Block::default()
                .borders(Borders::all())
                .title("Help")
                .title_style(Style::default().fg(app.theme.white))
                .border_style(Style::default().fg(app.theme.white))
                .padding(Padding::uniform(1));

            let paragraph =
                Paragraph::new(text).wrap(Wrap { trim: true }).block(block);

            frame.render_widget(Clear, popup_area);

            render_background(frame, popup_area, &app.theme);

            frame.render_widget(paragraph, popup_area);
        }
        false => {}
    }
}
