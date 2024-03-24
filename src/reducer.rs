use ratatui::widgets::ScrollbarState;
use tracing::info;

use crate::{
    actions::AppAction,
    app_state::{AppState, Screen},
};

fn should_skip_action(app_has_exception: bool, action: &AppAction) -> bool {
    if !app_has_exception {
        return false;
    }

    match action {
        AppAction::Quit | AppAction::ConfigurationLoaded(_) => false,
        _ => true,
    }
}

pub fn update_state(
    app: &mut AppState,
    action: AppAction,
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
            app.should_quit = true;
            None
        }
        AppAction::SetCriticalException(error) => {
            app.show_exception_screen(&error);
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
        AppAction::ScrollDiffsUp => {
            app.vertical_scroll = app.vertical_scroll.saturating_sub(5);
            app.vertical_scroll_state =
                app.vertical_scroll_state.position(app.vertical_scroll);
            None
        }
        AppAction::ScrollDiffsDown => {
            app.vertical_scroll = app.vertical_scroll.saturating_add(5);
            app.vertical_scroll_state =
                app.vertical_scroll_state.position(app.vertical_scroll);
            None
        }
        AppAction::GoToNextDiff => {
            app.go_to_next_diff();
            None
        }
        AppAction::GoToPreviousDiff => {
            app.go_to_prev_diff();
            None
        }
        AppAction::SetNotification(notification) => {
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
        AppAction::CloseDiffsScreen => {
            app.selected_job = None;
            app.current_screen = Screen::Home;
            Some(AppAction::ResetScrollState)
        }
        AppAction::ResetScrollState => {
            app.vertical_scroll = 0;
            app.vertical_scroll_state = ScrollbarState::default();
            None
        }
        AppAction::StartAllJobs => {
            app.reset_jobs_state();

            match app.current_screen {
                Screen::JobInfo => Some(AppAction::CloseDiffsScreen),
                _ => None,
            }
        }
        AppAction::StartOneJob(_) => match app.current_screen {
            Screen::JobInfo => Some(AppAction::CloseDiffsScreen),
            _ => None,
        },
        AppAction::SwitchDiffTab => {
            app.go_to_next_request_info_tab();

            Some(AppAction::ResetScrollState)
        }
        AppAction::ConfigurationLoaded(configuration) => {
            app.on_configuration_load(configuration);
            None
        }
        AppAction::LoadingJobsProgress(payload) => {
            app.on_load_jobs_progress_change(payload)
        }
        _ => None,
    }
}
