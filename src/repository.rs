
use std::collections::HashMap;

use crate::job::{Job, JobStatus};

pub enum JobRepository{
    MemJobRepository,
}

#[derive(Debug)]
pub(crate) struct MemJobRepository {
    jobs: HashMap<u32, Job>,
}

impl MemJobRepository {
    pub(crate) fn new() -> Self {
        Self { jobs: HashMap::new() }
    }

    pub(crate) fn add_job(&mut self, job: Job) {
        self.jobs.insert(job.id(), job);
    }

    pub(crate) fn get_job(&self, id: u32) -> Option<&Job> {
        self.jobs.get(&id)
    }

    pub(crate) fn get_job_mut(&mut self, id: u32) -> Option<&mut Job> {
        self.jobs.get_mut(&id)
    }

    pub(crate) fn get_next_executable_job(&mut self) -> Option<&mut Job> {
        self.jobs.values_mut().find(|job| matches!(job.status, JobStatus::Queued))
    }
}
