use crate::{
    actions::AppAction,
    http_diff::{
        config::{Configuration, DomainVariant},
        job::JobDTO,
        request::ResponseVariant,
        types::{AppError, JobStatus},
    },
    ui::{
        notification::{Notification, NotificationId, NotificationType},
        row::print_table_row,
        selected_job::SelectedJobState,
        table::get_headers_from_domains,
        theme::{get_dark_theme, get_light_theme, Theme, ThemeType},
        top::LOGO,
    },
};
use chrono::Local;
use crossterm::style::Stylize;
use ratatui::widgets::*;
use similar::ChangeTag;
use std::{
    cmp::{max, min},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tracing::{error, warn};

pub enum Screen {
    Home,
    JobInfo,
    Exception,
}

pub struct AppState {
    pub output_directory: PathBuf,
    pub state: TableState,
    pub jobs: Vec<JobDTO>,
    pub domains: Vec<String>,
    pub concurrency_level: usize,
    pub last_tick: Instant,
    pub content_length_downloaded_buffer: Vec<u64>,
    pub content_length_downloaded: Vec<u64>,
    pub notification: Option<Notification>,
    pub selected_job: Option<SelectedJobState>,
    pub should_show_help: bool,
    pub highlight_logo_row_index: usize,
    pub last_logo_change_color: Instant,
    pub should_quit: bool,
    pub critical_exception: Option<AppError>,
    pub current_screen: Screen,
    pub current_theme: ThemeType,
    pub theme: Theme,
    pub is_headless_mode: bool,
}

impl AppState {
    pub fn new(output_directory: &Path, is_headless_mode: bool) -> AppState {
        let started_at = Local::now();

        let base_directory = output_directory
            .join(started_at.format("%Y-%m-%d %H:%M:%S").to_string());

        AppState {
            output_directory: base_directory,
            state: TableState::default(),
            jobs: Vec::new(),
            domains: Vec::new(),
            concurrency_level: 0,
            content_length_downloaded: vec![0; 600],
            content_length_downloaded_buffer: Vec::new(),
            last_tick: Instant::now(),
            notification: None,
            selected_job: None,

            should_show_help: false,
            should_quit: false,
            highlight_logo_row_index: 0,
            last_logo_change_color: Instant::now(),
            current_screen: Screen::Home,
            current_theme: ThemeType::Dark,
            theme: get_dark_theme(),
            critical_exception: None,

            is_headless_mode,
        }
    }

    pub fn has_failed_jobs(&self) -> bool {
        self.jobs.iter().any(|job| job.is_failed())
    }

    pub fn change_a_theme(&mut self) {
        match self.current_theme {
            ThemeType::Dark => {
                self.current_theme = ThemeType::Light;
                self.theme = get_light_theme()
            }
            ThemeType::Light => {
                self.current_theme = ThemeType::Dark;
                self.theme = get_dark_theme()
            }
        }
    }

    pub fn set_critical_exception(&mut self, critical_exception: AppError) {
        self.critical_exception = Some(critical_exception);
    }

    pub fn select_next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.jobs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.jobs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_row_by_job_name(&mut self, job_name: &str) {
        if let Some(index) =
            self.jobs.iter().position(|job| job.job_name == job_name)
        {
            self.state.select(Some(index));
        }
    }

    pub fn set_selected_job(&mut self, mut job: JobDTO) -> Option<AppAction> {
        if job.status == JobStatus::Pending || job.status == JobStatus::Running
        {
            let notification = Notification::new(
                NotificationId::PendingJobInfoError,
                "The job is still executing. Please, wait",
                Some(Duration::from_secs(2)),
                NotificationType::Warning,
            );

            return Some(AppAction::SetNotification(notification));
        };

        if job.requests.is_empty() {
            warn!("set_selected_job failed as job doesn't have any requests: {:?}", job);
            return None;
        }

        self.current_screen = Screen::JobInfo;
        self.select_row_by_job_name(&job.job_name);

        let tab_index = 0;

        job.requests.sort_unstable_by(|a, b| b.has_diffs.cmp(&a.has_diffs));

        self.selected_job = Some(SelectedJobState::new(job, tab_index));

        None
    }

    pub fn get_current_job(&self) -> Option<JobDTO> {
        match self.state.selected() {
            Some(i) => match self.jobs.get(i) {
                Some(job) => Some(job.clone()),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_current_selected_row_index(&self) -> usize {
        match self.state.selected() {
            Some(i) => i,
            None => 0,
        }
    }

    pub fn get_previous_failed_job(&self) -> Option<JobDTO> {
        let start_index = self.get_current_selected_row_index();

        let (left, right) = self.jobs.split_at(start_index);

        if let Some(index) = left.iter().rposition(|job| job.is_failed()) {
            return self.jobs.get(index).and_then(|job| Some(job.clone()));
        } else if let Some(index) =
            right.iter().rposition(|job| job.is_failed())
        {
            return self
                .jobs
                .get(start_index + index)
                .and_then(|job| Some(job.clone()));
        }

        None
    }

    pub fn get_next_failed_job(&self) -> Option<JobDTO> {
        let start_index = self.get_current_selected_row_index() + 1;

        let (left, right) = self.jobs.split_at(start_index);

        if let Some(index) = right.iter().position(|job| job.is_failed()) {
            return self
                .jobs
                .get(start_index + index)
                .and_then(|job| Some(job.clone()));
        } else if let Some(index) = left.iter().position(|job| job.is_failed())
        {
            return self.jobs.get(index).and_then(|job| Some(job.clone()));
        }

        None
    }

    pub fn get_failed_jobs(&self) -> Vec<JobDTO> {
        let failed_jobs: Vec<JobDTO> = self
            .jobs
            .iter()
            .filter(|job| job.is_failed())
            .map(|job| job.to_owned())
            .collect();

        failed_jobs
    }

    pub fn upsert_jobs(&mut self, updated_jobs: Vec<JobDTO>) {
        for updated_job in updated_jobs {
            if self.is_headless_mode {
                match &updated_job.status {
                    JobStatus::Failed | JobStatus::Finished => {
                        let name = updated_job.job_name.clone();
                        let mut cells = vec![name.white()];

                        for request in updated_job.requests.iter() {
                            let raw_status_text =
                                request.get_status_text().trim().to_owned();

                            let text = match request.status {
                                JobStatus::Finished => {
                                    raw_status_text.green().bold()
                                }
                                JobStatus::Failed => {
                                    raw_status_text.white().on_red().bold()
                                }
                                _ => raw_status_text.white(),
                            };

                            cells.push(text);
                        }

                        print_table_row(cells, false);
                    }
                    _ => {}
                }
            }

            self.append_job_content_length_vec(&updated_job);

            if let Some(existing_job_dto) = self
                .jobs
                .iter_mut()
                .find(|dto| &dto.job_name == &updated_job.job_name)
            {
                *existing_job_dto = updated_job;
            } else {
                self.jobs.push(updated_job);
            }
        }
    }

    pub fn append_job_content_length_vec(&mut self, updated_job: &JobDTO) {
        let new_values: Vec<u64> = updated_job
            .requests
            .iter()
            .map(|request| match &request.response {
                Some(ResponseVariant::Success(response_variant)) => {
                    response_variant.content_length.unwrap_or_else(|| 0)
                }
                _ => 0,
            })
            .filter(|value| value != &0)
            .collect();

        for value in new_values {
            self.content_length_downloaded_buffer.push(value);
        }
    }

    pub fn clear_out_buffered_downloaded_data_indication(&mut self) {
        let value_to_insert =
            self.content_length_downloaded_buffer.iter().sum();

        self.content_length_downloaded_buffer.clear();

        self.assign_value_to_last_vec_element(value_to_insert);

        self.shift_content_length_vec(false);
    }

    pub fn assign_value_to_last_vec_element(&mut self, new_value: u64) {
        if let Some(old_value) = self.content_length_downloaded.get_mut(0) {
            *old_value = new_value;
        }
    }

    pub fn shift_content_length_vec(&mut self, should_pad_with_zero: bool) {
        if !self.content_length_downloaded.is_empty() {
            self.content_length_downloaded.rotate_right(1);

            if should_pad_with_zero {
                self.assign_value_to_last_vec_element(0);
            }
        }
    }

    pub fn on_tick(&mut self) {
        let now = Instant::now();
        let elapsed = (now - self.last_tick).as_secs_f64();
        if elapsed >= 0.2 {
            self.last_tick = now;

            self.clear_out_buffered_downloaded_data_indication();

            self.check_and_clear_expired_notification();
        }

        let elapsed_since_logo_color_change =
            (now - self.last_logo_change_color).as_secs_f64();
        if elapsed_since_logo_color_change >= 0.8 {
            self.last_logo_change_color = now;
            self.highlight_logo_row_index =
                (self.highlight_logo_row_index + 1) % LOGO.len();
        }
    }

    pub fn set_notification(&mut self, notification: Notification) {
        self.notification = Some(notification)
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    pub fn on_configuration_load(&mut self, configuration: Configuration) {
        self.domains = configuration
            .domains
            .iter()
            .map(|domain_variant| match domain_variant {
                DomainVariant::Url(domain) => domain.to_string(),
                DomainVariant::UrlWithHeaders(domain_config) => {
                    domain_config.domain.to_string()
                }
            })
            .collect();

        self.concurrency_level = configuration.concurrent_jobs;
        self.critical_exception = None;

        self.reset_jobs_state();

        if self.is_headless_mode {
            let table_headers = get_headers_from_domains(&self.domains)
                .iter()
                .map(|text| text.to_string().bold())
                .collect();

            print_table_row(table_headers, true);
        }
    }

    pub fn on_load_jobs_progress_change(
        &self,
        (current, total): (usize, usize),
    ) -> Option<AppAction> {
        let is_notification_displayed_currently =
            self.notification.as_ref().is_some();

        let are_sizes_different = current != total;

        let displayed_notification_id_matches =
            self.notification.as_ref().is_some_and(|notification| {
                notification.id == NotificationId::JobProgressChange
            });

        let should_issue_notification = (!is_notification_displayed_currently
            && are_sizes_different)
            || displayed_notification_id_matches;

        if should_issue_notification {
            let notification = Notification::new(
                NotificationId::JobProgressChange,
                &format!("Mapped {} out of {} requests.", current, total),
                Some(Duration::from_secs(2)),
                NotificationType::Success,
            );

            return Some(AppAction::SetNotification(notification));
        }

        None
    }

    pub fn reset_jobs_state(&mut self) {
        self.jobs.clear();
        self.selected_job = None;
        self.state = TableState::default();
        self.jobs = Vec::new();
        self.current_screen = Screen::Home;
    }

    pub fn reset_selected_job(&mut self) {
        self.selected_job = None;
    }

    pub fn find_next_diff_group(
        start_index: usize,
        diffs: &Vec<(ChangeTag, String)>,
        is_reversed_search: bool,
    ) -> Option<usize> {
        let last_element_index = diffs.len().saturating_sub(1);

        let mut current_index = min(max(start_index, 0), last_element_index);

        loop {
            let current_element = &diffs[current_index];
            let prev_index = if current_index == 0 {
                last_element_index
            } else {
                current_index.saturating_sub(1)
            };

            let prev_element = &diffs[prev_index];

            if current_element.0 != ChangeTag::Equal
                && prev_element.0 == ChangeTag::Equal
            {
                return Some(current_index);
            }

            current_index = match is_reversed_search {
                false => (current_index + 1) % diffs.len(),
                true => {
                    if current_index == 0 {
                        last_element_index
                    } else {
                        current_index.saturating_sub(1)
                    }
                }
            };

            if current_index == start_index {
                break;
            }
        }

        None
    }

    pub fn go_to_next_diff(&mut self) {
        match self.selected_job.as_mut() {
            Some(state) => match state
                .job
                .requests
                .get(state.tab_index)
                .unwrap()
                .diffs
                .is_empty()
            {
                false => {
                    let next_diff_group: Option<usize> =
                        AppState::find_next_diff_group(
                            min(
                                state.vertical_scroll.saturating_add(1),
                                state
                                    .job
                                    .requests
                                    .get(state.tab_index)
                                    .unwrap()
                                    .diffs
                                    .len(),
                            ),
                            &state
                                .job
                                .requests
                                .get(state.tab_index)
                                .unwrap()
                                .diffs,
                            false,
                        );

                    if let Some(next_group_index) = next_diff_group {
                        state.vertical_scroll = next_group_index;

                        state.vertical_scroll_state = state
                            .vertical_scroll_state
                            .position(state.vertical_scroll);
                    };
                }
                true => {}
            },
            None => {}
        }
    }

    pub fn go_to_prev_diff(&mut self) {
        match self.selected_job.as_mut() {
            Some(state) => {
                match state
                    .job
                    .requests
                    .get(state.tab_index)
                    .unwrap()
                    .diffs
                    .is_empty()
                {
                    false => {
                        let prev_diff_group: Option<usize> =
                            AppState::find_next_diff_group(
                                max(
                                    state.vertical_scroll.saturating_sub(1),
                                    0,
                                ),
                                &state
                                    .job
                                    .requests
                                    .get(state.tab_index)
                                    .unwrap()
                                    .diffs,
                                true,
                            );

                        if let Some(prev_group_index) = prev_diff_group {
                            state.vertical_scroll = prev_group_index;

                            state.vertical_scroll_state = state
                                .vertical_scroll_state
                                .position(state.vertical_scroll);
                        };
                    }
                    true => {}
                }
            }
            None => {}
        }
    }

    pub fn check_and_clear_expired_notification(&mut self) {
        match &self.notification {
            Some(notification) => match notification.expire_duration {
                Some(expiry) => {
                    let now = Instant::now();

                    let diff = now - notification.started_at;

                    if diff >= expiry {
                        self.clear_notification();
                    }
                }
                None => {}
            },
            None => {}
        }
    }

    pub fn show_help_screen(&mut self) {
        self.should_show_help = true
    }

    pub fn show_exception_screen(&mut self) {
        self.current_screen = Screen::Exception
    }

    pub fn close_help_screen(&mut self) {
        self.should_show_help = false
    }

    pub fn go_to_next_request_info_tab(&mut self) {
        if self.selected_job.is_none() {
            return;
        }

        let selected_job_state = self.selected_job.as_mut().unwrap();

        let tabs_count = selected_job_state.job.requests.len();

        selected_job_state.tab_index =
            (selected_job_state.tab_index + 1) % tabs_count;

        selected_job_state.vertical_scroll = 0;
        selected_job_state.vertical_scroll_state = ScrollbarState::default();
    }

    pub fn scroll_up(&mut self) -> Option<AppAction> {
        match self.selected_job.as_mut() {
            Some(state) => state.scroll_up(),
            None => {}
        }

        None
    }

    pub fn scroll_down(&mut self) -> Option<AppAction> {
        match self.selected_job.as_mut() {
            Some(state) => state.scroll_down(),
            None => {}
        }

        None
    }

    pub fn set_should_quit(&mut self, should_quit: bool) {
        self.should_quit = should_quit
    }

    fn generate_default_config(&self, path: &str) -> Result<(), AppError> {
        let default_config = Configuration::default();

        let file_path = Path::new(path);

        default_config.save(file_path).map_err(|err| {
            error!("Failed to save configuration file: {}", err);
            AppError::Exception("Failed to save configuration file".into())
        })?;

        Ok(())
    }

    pub fn save_default_config(&self) -> Option<AppAction> {
        let path = "./configuration.json";

        let notification = match self.generate_default_config(path) {
            Ok(()) => {
                if self.critical_exception.is_some() {
                    Notification::new(
                        NotificationId::GenerateDefaultConfig,
                        &format!("Saved default configuration to {}\nPlease, reload application", path),
                        None,
                        NotificationType::Success,
                    )
                } else {
                    Notification::new(
                        NotificationId::GenerateDefaultConfig,
                        &format!("Saved default configuration to {}", path),
                        Some(Duration::from_secs(5)),
                        NotificationType::Success,
                    )
                }
            }
            Err(_) => Notification::new(
                NotificationId::GenerateDefaultConfigFailed,
                &format!("Failed to save default configuration to {}", path),
                None,
                NotificationType::Error,
            ),
        };

        Some(AppAction::SetNotification(notification))
    }
}
