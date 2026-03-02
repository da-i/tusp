use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::Duration;

use crate::executor::{JobExecutionFailure, JobExecutionResult, JobExecutor};
use crate::job::{Job, JobFailureReason, JobStatus};
use crate::repository::MemJobRepository;
use crate::scheduler::JobScheduler;

pub(crate) struct Daemon {
    ipc_socket: String,
    scheduler: JobScheduler,
    executor: JobExecutor,
    repo: MemJobRepository,
}

pub(crate) fn run_daemon(ipc_socket: &str) {
    Daemon::new(ipc_socket).run();
}

pub(crate) fn submit_job(ipc_socket: &str, command: &str) -> Result<(), String> {
    // Submits a job to the daemon via the IPC socket.

    let mut stream = UnixStream::connect(ipc_socket)
        .map_err(|error| format!("cannot connect to daemon socket {ipc_socket}: {error}"))?;

    stream
        .write_all(format!("{command}\n").as_bytes())
        .map_err(|error| format!("cannot send job: {error}"))?;

    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader
        .read_line(&mut response)
        .map_err(|error| format!("cannot read daemon response: {error}"))?;

    print!("{response}");
    Ok(())
}

impl Daemon {
    pub(crate) fn new(ipc_socket: &str) -> Self {
        Self {
            ipc_socket: ipc_socket.to_string(),
            scheduler: JobScheduler::new(1),
            executor: JobExecutor::new(),
            repo: MemJobRepository::new(),
        }
    }

    pub(crate) fn run(&mut self) {
        match fs::remove_file(&self.ipc_socket) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                eprintln!("Failed to remove stale socket {}: {error}", self.ipc_socket);
                return;
            }
        }

        let listener = UnixListener::bind(&self.ipc_socket).expect("Failed to bind daemon IPC socket");
        listener
            .set_nonblocking(true)
            .expect("Failed to set daemon IPC socket to non-blocking mode");
        println!("tusp daemon listening on {}", self.ipc_socket);

        loop {
            self.poll_ipc_requests(&listener);
            self.run_scheduler_tick();
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn poll_ipc_requests(&mut self, listener: &UnixListener) {
        loop {
            match listener.accept() {
                Ok((stream, _)) => self.handle_stream(stream),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => {
                    eprintln!("IPC connection error: {error}");
                    break;
                }
            }
        }
    }

    fn handle_stream(&mut self, mut stream: UnixStream) {
        let mut request = String::new();
        let mut reader = BufReader::new(&stream);

        if reader.read_line(&mut request).is_err() {
            let _ = stream.write_all(b"ERR invalid request\n");
            return;
        }

        let command = request.trim();
        if command.is_empty() {
            let _ = stream.write_all(b"ERR empty command\n");
            return;
        }
        let job_id = self.repo.next_job_id();

        self.repo.add_job(Job::new(job_id, command.to_string(), 3));

        let response = format!("OK queued job {job_id}\n");
        let _ = stream.write_all(response.as_bytes());
    }

    fn run_scheduler_tick(&mut self) {
        let running_jobs = self.repo.count_running_jobs();
        if !self.scheduler.can_schedule_more(running_jobs) {
            return;
        }

        let Some(job_id) = self.repo.get_next_queued_job_id() else {
            return;
        };

        let Some(job_ref) = self.repo.get_job(job_id) else {
            return;
        };

        let Some(node_name) = self.scheduler.select_node_for_job(job_ref, running_jobs) else {
            return;
        };

        self.repo.update_job_status(job_id, JobStatus::Running);

        let execution_result = match self.repo.get_job(job_id) {
            Some(job) => self.executor.execute(job),
            None => return,
        };

        match execution_result {
            Ok(JobExecutionResult::Success(status_code)) => {
                self.repo
                    .update_job_status(job_id, JobStatus::Success { status_code });
                println!("job {} on node {} => success ({})", job_id, node_name, status_code);
            }
            Ok(JobExecutionResult::Failure(JobExecutionFailure::InvalidExitCode(error_code))) => {
                self.repo.update_job_status(
                    job_id,
                    JobStatus::Failure {
                        reason: JobFailureReason::NonZeroExit {
                            error_code,
                            message: format!("command exited with status {}", error_code),
                        },
                    },
                );
                println!("job {} on node {} => failed ({})", job_id, node_name, error_code);
            }
            Ok(JobExecutionResult::Failure(other_failure)) => {
                self.repo.update_job_status(
                    job_id,
                    JobStatus::Failure {
                        reason: JobFailureReason::NonZeroExit {
                            error_code: -1,
                            message: format!("execution failure: {:?}", other_failure),
                        },
                    },
                );
                println!("job {} on node {} => failed ({:?})", job_id, node_name, other_failure);
            }
            Err(error) => {
                self.repo.update_job_status(
                    job_id,
                    JobStatus::Failure {
                        reason: JobFailureReason::NonZeroExit {
                            error_code: -1,
                            message: format!("execution error: {:?}", error),
                        },
                    },
                );
                println!("job {} on node {} => error ({:?})", job_id, node_name, error);
            }
        }
    }
}
