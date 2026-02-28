use crate::job::{Job, JobStatus};

#[derive(Debug)]

pub(crate) struct JobExecutor {
    // Fields for managing job execution, such as worker threads, queues, etc.
    shell: String,
}

#[derive(Debug)]
pub enum JobExecutionResult {
    Success(i32), // Exit code
    Failure(JobExecutionFailure),
}

#[derive(Debug)]
pub enum JobExecutionFailure {
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