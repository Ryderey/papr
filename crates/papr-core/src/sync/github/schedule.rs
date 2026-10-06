//! Local scheduling policy. These settings never enter a cloud snapshot.
use super::error;
use crate::error::CoreError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

const KEY: &str = "github_local_schedule";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Schedule {
    pub enabled: bool,
    pub upload_delay_secs: u32,
    pub cloud_interval_minutes: u32,
    pub background_interval_minutes: u32,
}
impl Default for Schedule {
    fn default() -> Self {
        Self {
            enabled: true,
            upload_delay_secs: 30,
            cloud_interval_minutes: 10,
            background_interval_minutes: 60,
        }
    }
}
impl Schedule {
    pub fn validate(&self) -> Result<(), CoreError> {
        if !matches!(self.upload_delay_secs, 10 | 30 | 60 | 120)
            || !matches!(self.cloud_interval_minutes, 5 | 10 | 15 | 30 | 60)
            || !matches!(self.background_interval_minutes, 15 | 30 | 60 | 120 | 360)
        {
            return Err(error("githubInvalidSchedule"));
        }
        Ok(())
    }
}
pub fn load(conn: &Connection) -> Result<Schedule, CoreError> {
    let value: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key=?1", [KEY], |r| {
            r.get(0)
        })
        .optional()
        .map_err(db)?;
    let schedule: Schedule = match value.filter(|v| !v.is_empty()) {
        Some(value) => serde_json::from_str(&value).map_err(|_| error("githubInvalidSchedule"))?,
        None => Schedule::default(),
    };
    schedule.validate()?;
    Ok(schedule)
}
pub fn save(conn: &Connection, schedule: &Schedule) -> Result<(), CoreError> {
    schedule.validate()?;
    let value = serde_json::to_string(schedule).map_err(|_| error("githubInvalidSchedule"))?;
    conn.execute("INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![KEY,value]).map_err(db)?;
    Ok(())
}

/// Timers query only small connection/settings rows, never article statistics.
pub fn due(conn: &Connection, background: bool) -> Result<bool, CoreError> {
    let schedule = load(conn)?;
    if !schedule.enabled {
        return Ok(false);
    }
    let delay = format!("-{} seconds", schedule.upload_delay_secs);
    let deadline = format!("-{} seconds", (schedule.upload_delay_secs * 2).max(60));
    let interval = format!(
        "-{} minutes",
        if background {
            schedule.background_interval_minutes
        } else {
            schedule.cloud_interval_minutes
        }
    );
    conn.query_row(
        "SELECT (last_error_code IS NULL OR last_error_code IN ('githubNetwork','githubRateLimited','githubSyncCancelled','githubConcurrentRetryLimit','githubSyncBusy'))
         AND (retry_at IS NULL OR julianday(retry_at)<=julianday('now'))
         AND (lease_until IS NULL OR julianday(lease_until)<=julianday('now'))
         AND (
           ((?4 AND EXISTS(SELECT 1 FROM github_outbox WHERE connection_id=github_connections.id))
             OR (NOT ?4 AND pending_since IS NOT NULL AND
                 (julianday(last_edit_at)<=julianday('now',?1) OR julianday(pending_since)<=julianday('now',?2))))
           AND (last_publish_at IS NULL OR julianday(last_publish_at)<=julianday('now','-60 seconds'))
           OR last_success_at IS NULL OR julianday(last_success_at)<=julianday('now',?3)
         )
         FROM github_connections WHERE active=1", params![delay,deadline,interval,background], |r| r.get(0)
    ).optional().map_err(db).map(|value| value.unwrap_or(false))
}
fn db(e: rusqlite::Error) -> CoreError {
    CoreError::Db(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preferences_validate_atomically_and_remain_local() {
        let mut conn = Connection::open_in_memory().unwrap();
        crate::db::migrate(&mut conn).unwrap();
        assert_eq!(load(&conn).unwrap(), Schedule::default());
        let manual = Schedule {
            enabled: false,
            ..Schedule::default()
        };
        save(&conn, &manual).unwrap();
        assert!(!due(&conn, false).unwrap());
        assert!(!due(&conn, true).unwrap());
        for invalid in [
            Schedule {
                upload_delay_secs: 1,
                ..manual.clone()
            },
            Schedule {
                cloud_interval_minutes: 1,
                ..manual.clone()
            },
            Schedule {
                background_interval_minutes: 5,
                ..manual.clone()
            },
        ] {
            assert!(save(&conn, &invalid).is_err());
            assert_eq!(load(&conn).unwrap(), manual);
        }
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM github_outbox", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
