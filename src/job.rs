use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub(crate) struct Job {
    id: u32,
    pub(crate) status: JobStatus,
    pub(crate) command: String,
    pub(crate) attempt: u32,
    pub(crate) max_attempts: u32,
    pub(crate) pid: Option<u32>,
    pub(crate) stdout_path: Option<String>,
    pub(crate) stderr_path: Option<String>,
    pub(crate) created_at: Option<u64>,
    pub(crate) updated_at: Option<u64>,
    pub(crate) finished_at: Option<u64>,
}

#[derive(Debug)]
pub(crate) enum JobStatus {
    Success { status_code: i32 },
    Queued,
    Running,
    Failure { reason: JobFailureReason },
    Cancelled { reason: String, by_user: i32 },
}

#[derive(Debug)]
pub(crate) enum JobFailureReason {
    NonZeroExit { error_code: i32, message: String },
    Timeout { duration: u64 },
}

impl Job {
    pub(crate) fn new(id: u32,  command: String, max_attempts: u32) -> Self {
        let now = now_unix_seconds();
        Self {
            id,
            status: JobStatus::Queued,
            command,
            attempt: 0,
            max_attempts,
            pid: None,
            stdout_path: None,
            stderr_path: None,
            created_at: Some(now),
            updated_at: Some(now),
            finished_at: None,
        }
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn set_status(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Some(now_unix_seconds());
        if self.is_terminal() {
            self.finished_at = self.updated_at;
        }
    }

    pub(crate) fn status_label(&self) -> &'static str {
        match self.status {
            JobStatus::Success { .. } => "SUCCESS",
            JobStatus::Queued => "QUEUED",
            JobStatus::Running => "RUNNING",
            JobStatus::Failure { .. } => "FAILURE",
            JobStatus::Cancelled { .. } => "CANCELLED",
        }
    }

    pub(crate) fn age_seconds(&self) -> u64 {
        let now = now_unix_seconds();

        let age = self.finished_at.unwrap_or(self.created_at.unwrap_or(now));
        
        now - age

    }
    pub(crate) fn runtime_seconds(&self) -> u64 {
        // Zero if not finished, otherwise the difference between finished_at and created_at
        match (self.created_at, self.finished_at) {
            (Some(created_at), Some(finished_at)) => finished_at.saturating_sub(created_at),
            _ => 0,
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Success { .. } | JobStatus::Failure { .. } | JobStatus::Cancelled { .. }
        )
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_creation() {
        let job = Job::new(1, "echo Hello World".to_string(), 3);
        assert_eq!(job.id, 1);
        assert_eq!(job.command, "echo Hello World");
        match job.status {
            JobStatus::Queued => {},
            _ => panic!("Expected Queued variant"),
        }
        assert_eq!(job.command, "echo Hello World");
    }
}

