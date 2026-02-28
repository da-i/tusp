mod job;
mod scheduler;
mod executor;

fn main() {
    println!("Hello, world!");
    let mut sced = scheduler::JobScheduler::new(1);
    let mut exec = executor::JobExecutor::new();
    let mut job = job::Job::new(1, "touch Hello_World.txt".to_string(), 3);
    sced.add_job(job);

    sced.get_job(0);
    let exec_job = sced.get_next_executable_job().expect("'Failed to get job'");
    let result = exec.execute(exec_job).expect("'Job exec failed'");
    
}
