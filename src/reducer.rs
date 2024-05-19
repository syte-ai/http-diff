use crossterm::style::Stylize;
use tokio::sync::broadcast::Sender;
use tracing::info;

use crate::{
    actions::AppAction,
    app_state::{AppState, Screen},
    ui::notification::NotificationId,
};

fn should_skip_action(app_has_exception: bool, action: &AppAction) -> bool {
    if !app_has_exception {
        return false;
    }

    match action {
        AppAction::Quit
        | AppAction::ConfigurationLoaded(_)
        | AppAction::GenerateDefaultConfiguration
        | AppAction::DismissNotification
        | AppAction::SaveFailedJobs(_)
        | AppAction::SetNotification(_) => false,
        _ => true,
    }
}

pub fn update_state(
    app: &mut AppState,
    action: AppAction,
    events_sender: &Sender<AppAction>,
) -> Option<AppAction> {
    let skip_action =
        should_skip_action(app.critical_exception.is_some(), &action);

    if skip_action {
        info!(
            "Can not handle action '{:?}' as critical exception occurred",
            action
        );
        return None;
    }
    match action {
        AppAction::Quit => {
            app.set_should_quit(true);
            None
        }
        AppAction::SetCriticalException(error) => {
            app.set_critical_exception(error);

            if app.is_headless_mode {
                app.set_should_quit(true);
            } else {
                app.show_exception_screen();
            }
            None
        }
        AppAction::ShowHelp => {
            app.show_help_screen();
            None
        }
        AppAction::CloseHelp => {
            app.close_help_screen();
            None
        }
        AppAction::SelectPreviousRow => {
            app.select_previous_row();
            None
        }
        AppAction::SelectNextRow => {
            app.select_next_row();
            None
        }
        AppAction::ScrollUp => app.scroll_up(),
        AppAction::ScrollDown => app.scroll_down(),
        AppAction::GoToNextDiff => {
            app.go_to_next_diff();
            None
        }
        AppAction::GoToPreviousDiff => {
            app.go_to_prev_diff();
            None
        }
        AppAction::SetNotification(notification) => {
            if app.is_headless_mode {
                match &notification.id {
                    &NotificationId::AllRequestsFinishedWithFails => {
                        app.set_critical_exception(
                            crate::http_diff::types::AppError::Exception(
                                notification.body.clone(),
                            ),
                        );

                        app.set_notification(notification);

                        let failed_jobs = app.get_failed_jobs();

                        if !failed_jobs.is_empty() {
                            let _ = events_sender
                                .send(AppAction::SaveFailedJobs(failed_jobs));
                        } else {
                            app.set_should_quit(true)
                        }

                        return None;
                    }
                    NotificationId::AllRequestsFinishedWithoutFails
                    | NotificationId::SavedJobs => {
                        println!("\n{}", notification.body.as_str().green());

                        app.set_should_quit(true)
                    }
                    _ => {}
                };
            }

            app.set_notification(notification);

            None
        }
        AppAction::DismissNotification => {
            app.clear_notification();
            None
        }
        AppAction::ChangeTheme => {
            app.change_a_theme();
            None
        }
        AppAction::JobsUpdated(updated_jobs) => {
            app.upsert_jobs(updated_jobs);
            None
        }
        AppAction::ShowJobInfo(job) => app.set_selected_job(job),
        AppAction::SelectRowByJobName(job_name) => {
            app.select_row_by_job_name(&job_name);
            None
        }
        AppAction::CloseJobInfoScreen => {
            app.selected_job = None;
            app.current_screen = Screen::Home;
            None
        }
        AppAction::StartAllJobs => {
            app.reset_jobs_state();

            match app.current_screen {
                Screen::JobInfo => Some(AppAction::CloseJobInfoScreen),
                _ => None,
            }
        }
        AppAction::StartOneJob(_) => match app.current_screen {
            Screen::JobInfo => Some(AppAction::CloseJobInfoScreen),
            _ => None,
        },
        AppAction::SwitchTab => {
            app.go_to_next_request_info_tab();

            None
        }
        AppAction::ConfigurationLoaded(configuration) => {
            app.on_configuration_load(configuration);
            None
        }
        AppAction::LoadingJobsProgress(payload) => {
            app.on_load_jobs_progress_change(payload)
        }
        AppAction::GenerateDefaultConfiguration => app.save_default_config(),
        _ => None,
    }
}
