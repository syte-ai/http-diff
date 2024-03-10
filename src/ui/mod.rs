use crate::app_state::{AppState, Screen};
use ratatui::prelude::*;

use self::background::render_background;
use self::exception::render_exception;
use self::help_screen::render_help_popup;
use self::notification::render_notification;
use self::progress::render_progress_block;
use self::selected_job::render_selected_job;
use self::table::render_jobs_table;
use self::top::{render_top_block, LOGO};

pub mod background;
pub mod exception;
pub mod help_screen;
pub mod notification;
pub mod progress;
pub mod selected_job;
pub mod table;
pub mod theme;
pub mod top;
pub mod utils;

pub fn ui(f: &mut Frame, app: &mut AppState) {
    let background_block_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(f.size());

    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min((LOGO.len() + 1) as u16),
            Constraint::Min(5),
            Constraint::Min(8),
        ])
        .split(f.size());

    render_background(f, background_block_area[0], &app.theme);

    render_top_block(f, rects[0], app);
    render_progress_block(f, rects[1], app);
    render_jobs_table(f, rects[2], app);

    match app.current_screen {
        Screen::Home => {}
        Screen::JobInfo => {
            render_selected_job(f, app);
        }
        Screen::Exception => {
            render_exception(f, app);
        }
    }

    render_help_popup(f, app);
    render_notification(f, app);
}
