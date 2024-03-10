use crate::{
    actions::AppAction,
    http_diff::{
        config::{Configuration, DomainVariant},
        job::JobDTO,
        request::ResponseVariant,
        types::AppError,
    },
    notification::{Notification, NotificationType},
    ui::{
        theme::{get_dark_theme, get_light_theme, Theme, ThemeType},
        top::LOGO,
    },
};
use chrono::Local;
use ratatui::widgets::*;
use similar::ChangeTag;
use std::{
    cmp::{max, min},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub enum Screen {
    Home,
    JobInfo,
    Exception,
}

pub struct SelectedJobState {
    pub tab_index: usize,
    pub job: JobDTO,
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
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub selected_job: Option<SelectedJobState>,
    pub should_show_help: bool,
    pub highlight_logo_row_index: usize,
    pub last_logo_change_color: Instant,
    pub should_quit: bool,
    pub critical_exception: Option<AppError>,
    pub current_screen: Screen,
    pub current_theme: ThemeType,
    pub theme: Theme,
}

impl AppState {
    pub fn new(output_directory: &Path) -> AppState {
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
            vertical_scroll_state: ScrollbarState::default(),
            vertical_scroll: 0,
            should_show_help: false,
            should_quit: false,
            highlight_logo_row_index: 0,
            last_logo_change_color: Instant::now(),
            current_screen: Screen::Home,
            current_theme: ThemeType::Dark,
            theme: get_dark_theme(),
            critical_exception: None,
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

    pub fn upsert_jobs(&mut self, updated_jobs: &Vec<JobDTO>) {
        for updated_job in updated_jobs {
            if let Some(existing_job_dto) = self
                .jobs
                .iter_mut()
                .find(|dto| &dto.job_name == &updated_job.job_name)
            {
                *existing_job_dto = updated_job.clone();
            } else {
                self.jobs.push(updated_job.clone());
            }

            self.append_job_content_length_vec(&updated_job);
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

    pub fn set_notification(&mut self, notification: &Notification) {
        self.notification = Some(notification.clone())
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    pub fn on_configuration_load(&mut self, configuration: &Configuration) {
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

        self.reset_jobs_state()
    }

    pub fn on_load_jobs_progress_change(
        &self,
        (current, total): (usize, usize),
    ) -> Option<AppAction> {
        let notification_id = "jobs-progress-change";

        let should_issue_notification = self.notification.as_ref().is_none()
            || self.notification.as_ref().is_some_and(|notification| {
                notification.id == notification_id
            });

        if should_issue_notification {
            let notification = Notification::new(
                notification_id,
                &format!(
                    "Mapped {} requests out of {}. Please wait",
                    current, total
                ),
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
        self.vertical_scroll_state = ScrollbarState::default();
        self.vertical_scroll = 0;
        self.current_screen = Screen::Home;
    }

    pub fn reset_selected_job_diffs(&mut self) {
        self.selected_job = None;
        self.vertical_scroll_state = ScrollbarState::default();
        self.vertical_scroll = 0;
    }

    pub fn find_next_diff_group(
        &self,
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
        match &self.selected_job {
            Some(state) => match state
                .job
                .requests
                .get(1)
                .unwrap()
                .diffs
                .is_empty()
            {
                false => {
                    let next_diff_group: Option<usize> = self
                        .find_next_diff_group(
                            min(
                                self.vertical_scroll.saturating_add(1),
                                state.job.requests.get(1).unwrap().diffs.len(),
                            ),
                            &state.job.requests.get(1).unwrap().diffs,
                            false,
                        );

                    if let Some(next_group_index) = next_diff_group {
                        self.vertical_scroll = next_group_index;
                        self.vertical_scroll_state = self
                            .vertical_scroll_state
                            .position(self.vertical_scroll);
                    };
                }
                true => {}
            },
            None => {}
        }
    }

    pub fn go_to_prev_diff(&mut self) {
        match &self.selected_job {
            Some(state) => {
                match state.job.requests.get(1).unwrap().diffs.is_empty() {
                    false => {
                        let prev_diff_group: Option<usize> = self
                            .find_next_diff_group(
                                max(self.vertical_scroll.saturating_sub(1), 0),
                                &state.job.requests.get(1).unwrap().diffs,
                                true,
                            );

                        if let Some(prev_group_index) = prev_diff_group {
                            self.vertical_scroll = prev_group_index;
                            self.vertical_scroll_state = self
                                .vertical_scroll_state
                                .position(self.vertical_scroll);
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

    pub fn show_exception_screen(&mut self, error: &AppError) {
        self.critical_exception = Some(error.clone());
        self.current_screen = Screen::Exception
    }

    pub fn close_help_screen(&mut self) {
        self.should_show_help = false
    }

    pub fn go_to_next_diff_tab(&mut self) {
        if let Some(state) = &mut self.selected_job {
            let tabs_count = state.job.get_requests_with_diffs().len();

            state.tab_index = (state.tab_index + 1) % tabs_count;
        }
    }
}
