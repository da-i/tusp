mod job;
mod scheduler;
mod executor;
mod repository;

fn main() {
    println!("Hello, world!");
    let mut sced = scheduler::JobScheduler::new(1);
    let mut exec = executor::JobExecutor::new();
    let mut job = job::Job::new(1, "touch Hello_World.txt".to_string(), 3);
    let mut repo = repository::MemJobRepository::new();
    repo.add_job(job);
    

    let exec_job = repo.get_next_executable_job().expect("'Failed to get job'");
    let result = exec.execute(exec_job).expect("'Job exec failed'");
    println!("Job executed with result: {:?}", result);    
}
