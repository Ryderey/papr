//! Explicit, secret-free GitHub synchronization bridge DTOs.
use flutter_rust_bridge::frb;
use papr_core::sync::github::{model, schedule, service};

#[derive(Debug, Clone)]
#[frb]
pub struct GithubSchedule {
    pub enabled: bool,
    pub upload_delay_secs: u32,
    pub cloud_interval_minutes: u32,
    pub background_interval_minutes: u32,
}
impl From<schedule::Schedule> for GithubSchedule {
    fn from(s: schedule::Schedule) -> Self {
        Self {
            enabled: s.enabled,
            upload_delay_secs: s.upload_delay_secs,
            cloud_interval_minutes: s.cloud_interval_minutes,
            background_interval_minutes: s.background_interval_minutes,
        }
    }
}
impl From<GithubSchedule> for schedule::Schedule {
    fn from(s: GithubSchedule) -> Self {
        Self {
            enabled: s.enabled,
            upload_delay_secs: s.upload_delay_secs,
            cloud_interval_minutes: s.cloud_interval_minutes,
            background_interval_minutes: s.background_interval_minutes,
        }
    }
}

#[derive(Debug, Clone)]
#[frb]
pub struct GithubProfile {
    pub repository_id: i64,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub credential_ref: String,
}
impl From<model::GitHubProfile> for GithubProfile {
    fn from(p: model::GitHubProfile) -> Self {
        Self {
            repository_id: p.repository_id as i64,
            owner: p.owner,
            repo: p.repo,
            branch: p.branch,
            credential_ref: p.credential_ref,
        }
    }
}
impl From<GithubProfile> for model::GitHubProfile {
    fn from(p: GithubProfile) -> Self {
        Self {
            repository_id: p.repository_id as u64,
            owner: p.owner,
            repo: p.repo,
            branch: p.branch,
            credential_ref: p.credential_ref,
        }
    }
}
#[derive(Debug, Clone)]
#[frb]
pub struct GithubPreview {
    pub profile: GithubProfile,
    pub head: String,
    pub local_feeds: i64,
    pub remote_feeds: i64,
    pub local_articles: i64,
    pub remote_articles: i64,
    pub excluded_feeds: i64,
    pub excluded_articles: i64,
    pub warning_count: i64,
}
impl From<service::Preview> for GithubPreview {
    fn from(p: service::Preview) -> Self {
        Self {
            profile: p.profile.into(),
            head: p.head,
            local_feeds: p.local_feeds as i64,
            remote_feeds: p.remote_feeds as i64,
            local_articles: p.local_articles as i64,
            remote_articles: p.remote_articles as i64,
            excluded_feeds: p.excluded_feeds as i64,
            excluded_articles: p.excluded_articles as i64,
            warning_count: p.warning_count as i64,
        }
    }
}
impl From<GithubPreview> for service::Preview {
    fn from(p: GithubPreview) -> Self {
        Self {
            profile: p.profile.into(),
            head: p.head,
            local_feeds: p.local_feeds.max(0) as usize,
            remote_feeds: p.remote_feeds.max(0) as usize,
            local_articles: p.local_articles.max(0) as usize,
            remote_articles: p.remote_articles.max(0) as usize,
            excluded_feeds: p.excluded_feeds.max(0) as usize,
            excluded_articles: p.excluded_articles.max(0) as usize,
            warning_count: p.warning_count.max(0) as usize,
        }
    }
}
#[derive(Debug, Clone)]
#[frb]
pub struct GithubStatus {
    pub profile: Option<GithubProfile>,
    pub pending: i64,
    pub rejected: i64,
    pub metadata_only: i64,
    pub last_success_at: Option<String>,
    pub last_error_code: Option<String>,
    pub retry_at: Option<String>,
    pub busy: bool,
    pub uncertain_publication: bool,
    pub background_due: bool,
    pub automatic_due: bool,
}
impl From<service::Status> for GithubStatus {
    fn from(s: service::Status) -> Self {
        Self {
            profile: s.profile.map(Into::into),
            pending: s.pending as i64,
            rejected: s.rejected as i64,
            metadata_only: s.metadata_only as i64,
            last_success_at: s.last_success_at,
            last_error_code: s.last_error_code,
            retry_at: s.retry_at,
            busy: s.busy,
            uncertain_publication: s.uncertain_publication,
            background_due: s.background_due,
            automatic_due: s.automatic_due,
        }
    }
}
#[derive(Debug, Clone)]
#[frb]
pub struct GithubSyncReport {
    pub unchanged: bool,
    pub acknowledged: i64,
    pub rejected: i64,
    pub retries: i64,
    pub pending: i64,
}
impl From<service::SyncReport> for GithubSyncReport {
    fn from(s: service::SyncReport) -> Self {
        Self {
            unchanged: s.unchanged,
            acknowledged: s.acknowledged as i64,
            rejected: s.rejected as i64,
            retries: s.retries as i64,
            pending: s.pending as i64,
        }
    }
}
