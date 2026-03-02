
use std::collections::HashMap;

use crate::job::{Job, JobStatus};

pub enum JobRepository{
    MemJobRepository,
}

#[derive(Debug)]
pub(crate) struct MemJobRepository {
    jobs: HashMap<u32, Job>,
    next_job_id: u32,
}

impl MemJobRepository {
    pub(crate) fn new() -> Self {
        Self { jobs: HashMap::new(), next_job_id: 1 }
    }
    pub(crate) fn next_job_id(&mut self) -> u32 {
        let current_id = self.next_job_id;
        self.next_job_id += 1;
        current_id
    }

    pub(crate) fn add_job(&mut self, job: Job) {
        self.jobs.insert(job.id(), job);
    }

    pub(crate) fn get_job(&self, id: u32) -> Option<&Job> {
        self.jobs.get(&id)
    }

    pub(crate) fn get_next_queued_job_id(&self) -> Option<u32> {
        self.jobs
            .iter()
            .find_map(|(job_id, job)| matches!(job.status, JobStatus::Queued).then_some(*job_id))
    }

    pub(crate) fn count_running_jobs(&self) -> usize {
        self.jobs
            .values()
            .filter(|job| matches!(job.status, JobStatus::Running))
            .count()
    }

    pub(crate) fn list_jobs(&self) -> Vec<&Job> {
        let mut jobs: Vec<&Job> = self.jobs.values().collect();
        jobs.sort_by_key(|job| job.id());
        jobs
    }

    pub(crate) fn update_job_status(&mut self, job_id: u32, new_status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.set_status(new_status);
            // Update timestamps as needed
        }
    }
}
