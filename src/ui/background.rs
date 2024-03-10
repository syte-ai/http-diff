use ratatui::{prelude::*, widgets::*};

use super::theme::Theme;

pub fn render_background(frame: &mut Frame, area: Rect, theme: &Theme) {
    let background_block = Block::default().bg(theme.background);

    frame.render_widget(background_block, area);
}
