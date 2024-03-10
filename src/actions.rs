use std::time::Duration;

use crossterm::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind,
};
use similar::ChangeTag;

use crate::{
    app_state::{AppState, Screen},
    http_diff::{
        config::Configuration,
        job::JobDTO,
        types::{AppError, JobStatus},
    },
    notification::{Notification, NotificationType},
};

#[derive(Debug, Clone)]
pub struct JobDiffs {
    pub diffs: Vec<(ChangeTag, String)>,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]

pub enum AppAction {
    Quit,

    ReloadConfigurationFile(String),
    TryLoadConfigurationFile(String),
    ConfigurationLoaded(Configuration),

    LoadingJobsProgress((usize, usize)),

    SetCriticalException(AppError),

    StartAllJobs,
    StartOneJob(String),
    ResetScrollState,

    ShowHelp,
    CloseHelp,

    SelectRowByJobName(String),
    SelectPreviousRow,
    SelectNextRow,

    SwitchDiffTab,
    ScrollDiffsUp,
    ScrollDiffsDown,
    GoToNextDiff,
    GoToPreviousDiff,

    SetNotification(Notification),
    DismissNotification,

    ChangeTheme,

    JobsUpdated(Vec<JobDTO>),

    ShowJobInfo(JobDTO),
    CloseDiffsScreen,

    SaveFailedJobs(Vec<JobDTO>),
    SaveCurrentJob(JobDTO),
}

pub fn event_to_app_action(
    event: &Event,
    app: &AppState,
) -> Option<AppAction> {
    match event {
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => Some(AppAction::Quit),
                    KeyCode::Char('h') => Some(AppAction::ShowHelp),
                    KeyCode::Char('t') => Some(AppAction::ChangeTheme),
                    KeyCode::Char('R') => Some(AppAction::StartAllJobs),
                    KeyCode::Char('r') => match app.get_current_job() {
                        Some(job) => {
                            Some(AppAction::StartOneJob(job.job_name))
                        }
                        None => None,
                    },
                    KeyCode::Enter => match app.get_current_job() {
                        Some(job) => match job.status {
                            JobStatus::Failed => {
                                Some(AppAction::ShowJobInfo(job))
                            }
                            _ => None,
                        },
                        None => None,
                    },
                    KeyCode::Up => match app.current_screen {
                        Screen::Home => match key.modifiers {
                            KeyModifiers::SHIFT => {
                                if let Some(previous_failed) =
                                    app.get_previous_failed_job()
                                {
                                    return Some(
                                        AppAction::SelectRowByJobName(
                                            previous_failed.job_name,
                                        ),
                                    );
                                }
                                None
                            }
                            KeyModifiers::NONE => {
                                Some(AppAction::SelectPreviousRow)
                            }
                            _ => None,
                        },
                        Screen::JobInfo => match key.modifiers {
                            KeyModifiers::SHIFT => {
                                Some(AppAction::GoToPreviousDiff)
                            }
                            KeyModifiers::NONE => {
                                Some(AppAction::ScrollDiffsUp)
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    KeyCode::Down => match app.current_screen {
                        Screen::Home => match key.modifiers {
                            KeyModifiers::SHIFT => {
                                if let Some(next_failed) =
                                    app.get_next_failed_job()
                                {
                                    return Some(
                                        AppAction::SelectRowByJobName(
                                            next_failed.job_name,
                                        ),
                                    );
                                }
                                None
                            }
                            KeyModifiers::NONE => {
                                Some(AppAction::SelectNextRow)
                            }
                            _ => None,
                        },
                        Screen::JobInfo => match key.modifiers {
                            KeyModifiers::SHIFT => {
                                Some(AppAction::GoToNextDiff)
                            }
                            KeyModifiers::NONE => {
                                Some(AppAction::ScrollDiffsDown)
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    KeyCode::Right => match app.current_screen {
                        Screen::Home => None,
                        Screen::JobInfo => match key.modifiers {
                            KeyModifiers::SHIFT => {
                                if let Some(next_failed) =
                                    app.get_next_failed_job()
                                {
                                    return Some(AppAction::ShowJobInfo(
                                        next_failed,
                                    ));
                                }
                                None
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    KeyCode::Left => match app.current_screen {
                        Screen::Home => None,
                        Screen::JobInfo => match key.modifiers {
                            KeyModifiers::SHIFT => {
                                if let Some(previous_failed) =
                                    app.get_previous_failed_job()
                                {
                                    return Some(AppAction::ShowJobInfo(
                                        previous_failed,
                                    ));
                                }
                                None
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    KeyCode::Tab => match app.current_screen {
                        Screen::JobInfo => Some(AppAction::SwitchDiffTab),
                        _ => None,
                    },
                    KeyCode::Esc => {
                        if app.should_show_help {
                            Some(AppAction::CloseHelp)
                        } else {
                            match app.selected_job {
                                None => Some(AppAction::DismissNotification),
                                _ => Some(AppAction::CloseDiffsScreen),
                            }
                        }
                    }
                    KeyCode::Char('S') => {
                        let failed_jobs = app.get_failed_jobs();

                        if failed_jobs.is_empty() {
                            Some(AppAction::SetNotification(
                                Notification::new(
                                    "no-failed-jobs-to-save",
                                    "There are no failed jobs to save",
                                    Some(Duration::from_secs(5)),
                                    NotificationType::Warning,
                                ),
                            ))
                        } else {
                            Some(AppAction::SaveFailedJobs(failed_jobs))
                        }
                    }
                    KeyCode::Char('s') => match app.get_current_job() {
                        Some(job) => Some(AppAction::SaveCurrentJob(job)),
                        None => None,
                    },
                    _ => None,
                }
            } else {
                None
            }
        }
        Event::Mouse(mouse_event) => match mouse_event.kind {
            MouseEventKind::ScrollUp => match app.current_screen {
                Screen::Home => Some(AppAction::SelectPreviousRow),
                Screen::JobInfo => Some(AppAction::ScrollDiffsUp),
                _ => None,
            },
            MouseEventKind::ScrollDown => match app.current_screen {
                Screen::Home => Some(AppAction::SelectNextRow),
                Screen::JobInfo => Some(AppAction::ScrollDiffsDown),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}
