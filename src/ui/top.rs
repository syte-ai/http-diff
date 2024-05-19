use crate::http_diff::config::APP_VERSION;
use crate::http_diff::types::JobStatus;
use crate::{app_state::AppState, http_diff::config::APP_NAME};
use ratatui::{prelude::*, widgets::*};

pub const LOGO: [&str; 6] = [
    "",
    "█▄█ ▀█▀ ▀█▀ █▀▄",
    "█ █  █   █  █▀ ",
    "",
    "█▀▄  █  █▀  █▀",
    "█▄▀  █  █▀  █▀",
];

pub fn render_top_block(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    render_welcome_block(frame, chunks[0], app);
    render_status_bars(frame, chunks[1], app);
}

fn render_status_bars(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let bar_chart_block = Block::default()
        .title("status")
        .borders(Borders::ALL)
        .title_style(Style::default().fg(app.theme.white))
        .border_style(Style::default().fg(app.theme.white))
        .padding(Padding::horizontal(1));

    let label_style = Style::default().fg(app.theme.white);

    let bar_group = create_status_bar_groups(app);

    let group_gap = 8;
    let bar_width = ((area.width - 2) - (group_gap / 2) - 1) / 4;

    let barchart = BarChart::default()
        .block(bar_chart_block)
        .bar_width(bar_width)
        .data(bar_group)
        .group_gap(group_gap)
        .label_style(label_style);

    frame.render_widget(barchart, area);
}

fn render_welcome_block(frame: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(17), Constraint::Percentage(100)])
        .split(area);

    render_logo(frame, chunks[0], app);
    render_welcome_text(frame, chunks[1], app);
}

fn render_logo(frame: &mut Frame, area: Rect, app: &AppState) {
    let area = area.inner(&Margin { vertical: 0, horizontal: 1 });

    let mut text: Vec<Line> = LOGO
        .iter()
        .map(|text| Line::from((*text).fg(app.theme.white)))
        .collect();

    let highlighted_row_color = if app.has_failed_jobs() {
        app.theme.warning
    } else {
        app.theme.success
    };

    text.get_mut(app.highlight_logo_row_index)
        .unwrap()
        .patch_style(Style::default().fg(highlighted_row_color));

    text.push(Line::from(
        format!("v{}", APP_VERSION).italic().fg(app.theme.white),
    ));

    let paragraph = Paragraph::new(text);

    frame.render_widget(paragraph, area);
}

fn render_welcome_text(frame: &mut Frame, area: Rect, app: &AppState) {
    let paragraph = keys_explanation_paragraph(app);

    let block = Block::new()
        .borders(Borders::ALL)
        .title(APP_NAME)
        .title_style(Style::default().fg(app.theme.white))
        .border_style(Style::default().fg(app.theme.white))
        .padding(Padding::new(1, 1, 1, 0));

    frame.render_widget(paragraph.clone().block(block), area);
}

fn create_status_bar_groups(app: &mut AppState) -> BarGroup {
    let mut data = vec![
        ("Pending", 0, Style::default().fg(app.theme.gray)),
        ("Running", 0, Style::default().fg(app.theme.warning)),
        ("Failed", 0, Style::default().fg(app.theme.error)),
        ("Success", 0, Style::default().fg(app.theme.success)),
    ];

    for job in &app.jobs {
        match job.status {
            JobStatus::Pending => data[0].1 += 1,
            JobStatus::Running => data[1].1 += 1,
            JobStatus::Failed => data[2].1 += 1,
            JobStatus::Finished => data[3].1 += 1,
        }
    }

    let bars: Vec<Bar> = data
        .iter()
        .map(|c| {
            Bar::default()
                .value(c.1)
                .style(c.2)
                .label(c.0.into())
                .text_value(c.1.to_string())
                .value_style(
                    Style::default()
                        .bg(c.2.fg.unwrap())
                        .fg(app.theme.background),
                )
        })
        .collect();

    BarGroup::default().bars(&bars)
}

fn keys_explanation_paragraph(app: &AppState) -> Paragraph<'static> {
    let text = vec![
        Line::from("Welcome,".italic().bold().fg(app.theme.white)),
        Line::from(
            "press `h` to show help screen".italic().fg(app.theme.white),
        ),
        Line::from(vec![
            "concurrent requests: ".italic().fg(app.theme.white),
            Span::styled(
                app.concurrency_level.to_string(),
                Style::default()
                    .fg(app.theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    Paragraph::new(text).wrap(Wrap { trim: true })
}

pub fn print_logo() {
    for line in LOGO.iter() {
        println!("{}", line)
    }

    println!("v{}\n", APP_VERSION)
}
