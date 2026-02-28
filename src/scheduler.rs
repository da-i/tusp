  use std::{process::Output, result};

use crate::job::{Job, JobStatus};

pub(crate) struct JobScheduler {
    max_jobs: usize,
    jobs: Vec<Job>,
}

impl JobScheduler {
    pub(crate) fn new(max_jobs: usize) -> Self {
        JobScheduler {
            max_jobs,
            jobs: Vec::new(),
        }
    }
 
    pub(crate) fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub(crate) fn get_job(&self, job_id: u32) -> Option<&Job> {
        self.jobs.iter().find(|job| job.id() == job_id)
    }

    pub(crate) fn update_job_status(&mut self, job_id: u32, new_status: JobStatus) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id() == job_id) {
            job.set_status(new_status);
            // Update timestamps as needed
        }
    }

    // Additional methods for scheduling, executing, and managing jobs can be added here
}

struct JobExecutor {
    // Fields for managing job execution, such as worker threads, queues, etc.
    shell: String,
}

pub enum JobExecutionResult {
    Success(i32), // Exit code
    Failure(JobExecutionFailure),
}

enum JobExecutionFailure {
    ShellNotFound,
    InvalidExitCode(i32),
    ExecutionFailed(String),
    JobStatusInvalid(String),
    // Other error variants can be added here
}

impl JobExecutor {
    pub(crate) fn new() -> Self {
        Self {
            shell: "/bin/bash".to_string()
        }
    }

    pub(crate) fn execute(&self, job: &Job) -> Result<JobExecutionResult, JobExecutionFailure> {
        // Logic to execute the job using the specified shell
        // This could involve spawning a process, handling output, etc.
        if !matches!(job.status, JobStatus::Queued) {
            return Err(JobExecutionFailure::JobStatusInvalid(
                format!("Job {:?} is not in a valid state for execution", job.id())
            ));
        }
        let result = std::process::Command::new(&self.shell)
            .arg("-c")
            .arg(&job.command)
            .output();

        match result {
            Ok(output) => {
                let exit_code = output.status.code().unwrap_or(-1);
                if output.status.success() {
                    Ok(JobExecutionResult::Success(exit_code))
                } else {
                    Ok(JobExecutionResult::Failure(JobExecutionFailure::InvalidExitCode(exit_code)))
                    
                }
            }
            Err(e) => Err(JobExecutionFailure::ExecutionFailed(
                format!("Failed to execute command: {}", e)
            )),
        }
    }

    // Additional methods for managing job execution can be added here
}