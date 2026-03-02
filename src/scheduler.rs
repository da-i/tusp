use crate::job::Job;

struct JobNode{
    name: String,
    capacity: usize,
}

pub(crate) struct JobScheduler {
    max_jobs: usize,
    nodes: Vec<JobNode>,
}

impl JobScheduler {
    pub(crate) fn new(max_jobs: usize) -> Self {
        JobScheduler {
            max_jobs,
            nodes: vec![JobNode { name: "default".to_string(), capacity: 1 }],
        }
    }

    pub(crate) fn can_schedule_more(&self, running_jobs: usize) -> bool {
        let cluster_capacity: usize = self.nodes.iter().map(|node| node.capacity).sum();
        let effective_capacity = self.max_jobs.min(cluster_capacity);
        running_jobs < effective_capacity
    }

    pub(crate) fn select_node_for_job(&self, _job: &Job, running_jobs: usize) -> Option<String> {
        if !self.can_schedule_more(running_jobs) {
            return None;
        }

        self.nodes
            .iter()
            .find(|node| node.capacity > 0)
            .map(|node| node.name.clone())
    }

    // Additional methods for scheduling, and managing nodes can be added here
}
