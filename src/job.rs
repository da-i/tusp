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
pub(crate) struct JobRepository {
    jobs: Vec<Job>,
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
        Self {
            id,
            status: JobStatus::Queued,
            command,
            attempt: 0,
            max_attempts,
            pid: None,
            stdout_path: None,
            stderr_path: None,
            created_at: None,
            updated_at: None,
            finished_at: None,
        }
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn set_status(&mut self, status: JobStatus) {
        self.status = status;
    }
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

