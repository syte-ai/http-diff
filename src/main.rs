use actions::{event_to_app_action, AppAction};
use anyhow::Result;
use app_state::AppState;
use chrono::Local;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use http_diff::app::App as HttpDiff;
use ratatui::prelude::*;
use reducer::update_state;
use std::{error::Error, io, time::Duration};
use std::{fs::File, sync::Arc};
use std::{path::Path, time::Instant};
use tokio::sync::broadcast;
use tracing::error;
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt,
    prelude::*,
};
use ui::ui;
use worker::{
    get_configuration_file_watcher, handle_commands_to_http_diff_loop,
    process_app_action,
};

pub mod actions;
pub mod app_state;
pub mod cli;
pub mod http_diff;
pub mod reducer;
pub mod ui;
pub mod worker;
use cli::Arguments;

pub fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen
        )
        .unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    initialize_panic_handler();
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

fn reset_terminal(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = match Arguments::try_parse() {
        Err(err) => {
            println!("{err}");
            return Ok(());
        }
        Ok(args) => args,
    };

    if args.enable_log {
        let _ = tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_writer(Arc::new(File::create("./.log")?))
                    .json()
                    .with_filter(
                        Targets::default()
                            .with_target("http_diff", LevelFilter::DEBUG),
                    ),
            )
            .try_init();
    }

    let mut terminal = init_terminal()?;

    let res = run_app(&mut terminal).await;

    reset_terminal(&mut terminal)?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let args = Arguments::try_parse()?;

    let output_directory = Path::new(&args.output_directory);

    let started_at = Local::now();

    let base_output_directory = output_directory
        .join(started_at.format("%Y-%m-%d %H:%M:%S").to_string());

    let mut app = AppState::new(&output_directory);

    let (event_loop_actions_sender, mut event_loop_actions_receiver) =
        broadcast::channel::<AppAction>(1000);

    let worker_actions_sender = event_loop_actions_sender.clone();

    let mut http_diff_actions_receiver = event_loop_actions_sender.subscribe();
    let mut worker_actions_receiver = worker_actions_sender.subscribe();

    let mut http_diff = HttpDiff::new(event_loop_actions_sender.clone())?;

    tokio::spawn(async move {
        loop {
            match handle_commands_to_http_diff_loop(
                &mut http_diff_actions_receiver,
                &mut http_diff,
            )
            .await
            {
                Ok(()) => break,
                Err(app_error) => {
                    error!("handle_commands_to_http_diff_loop returned error: {app_error}");

                    let _ = http_diff
                        .app_actions_sender
                        .send(AppAction::SetCriticalException(app_error));
                }
            }
        }
    });

    let app_actions_sender = event_loop_actions_sender.clone();
    let configuration_file_path = args.configuration.clone();

    let _watcher = get_configuration_file_watcher(
        configuration_file_path,
        app_actions_sender,
    )
    .await?;

    tokio::spawn(async move {
        loop {
            let action = match worker_actions_receiver.recv().await {
                Ok(action) => action,
                Err(_) => continue,
            };

            let output_dir = base_output_directory.clone();
            let sender = worker_actions_sender.clone();

            tokio::spawn(async move {
                process_app_action(&action, sender, &output_dir).await;
            });
        }
    });

    let event_timeout = Duration::from_millis(60);

    event_loop_actions_sender
        .send(AppAction::TryLoadConfigurationFile(args.configuration))?;

    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| ui(f, &mut app))?;

        let started_to_poll = Instant::now();

        // receive input and dispatch actions
        while started_to_poll.elapsed() < event_timeout {
            if event::poll(event_timeout - started_to_poll.elapsed())? {
                if let Some(action) =
                    event_to_app_action(&event::read()?, &app)
                {
                    event_loop_actions_sender.send(action)?;
                }
            } else {
                break;
            }
        }

        // receive and act on actions
        loop {
            match event_loop_actions_receiver.try_recv() {
                Ok(action) => {
                    let mut current_action = Some(action);

                    while let Some(a) = current_action {
                        current_action = update_state(&mut app, a);
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }

        app.on_tick();
    }
}
