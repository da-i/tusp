mod job;
mod scheduler;

fn main() {
    println!("Hello, world!");
    let mut sced = scheduler::JobScheduler::new(1);
    let mut job = job::Job::new(1, "echo Hello World".to_string(), 3);
    sced.add_job(job);

    
}
