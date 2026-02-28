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

    pub(crate) fn get_next_executable_job(&self) -> Option<&Job> {
        self.jobs.iter().find(|job| matches!(job.status, JobStatus::Queued))
    }

    pub(crate) fn update_job_status(&mut self, job_id: u32, new_status: JobStatus) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id() == job_id) {
            job.set_status(new_status);
            // Update timestamps as needed
        }
    }

    // Additional methods for scheduling, executing, and managing jobs can be added here
}
