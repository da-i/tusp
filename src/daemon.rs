use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
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
    let response = send_request(ipc_socket, &format!("SUBMIT {command}\n"))?;
    print!("{response}");
    Ok(())
}

pub(crate) fn list_jobs(ipc_socket: &str) -> Result<(), String> {
    let response = send_request(ipc_socket, "LIST\n")?;
    print!("{response}");
    Ok(())
}

fn send_request(ipc_socket: &str, request: &str) -> Result<String, String> {
    let mut stream = UnixStream::connect(ipc_socket)
        .map_err(|error| format!("cannot connect to daemon socket {ipc_socket}: {error}"))?;

    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("cannot send request: {error}"))?;

    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|error| format!("cannot finish request: {error}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("cannot read daemon response: {error}"))?;

    Ok(response)
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

        let request = request.trim();
        if request.is_empty() {
            let _ = stream.write_all(b"ERR empty command\n");
            return;
        }

        if request == "LIST" {
            let response = self.format_jobs_table();
            let _ = stream.write_all(response.as_bytes());
            return;
        }

        let command = request.strip_prefix("SUBMIT ").unwrap_or(request).trim();
        if command.is_empty() {
            let _ = stream.write_all(b"ERR empty command\n");
            return;
        }

        let job_id = self.repo.next_job_id();

        self.repo.add_job(Job::new(job_id, command.to_string(), 3));

        let response = format!("OK queued job {job_id}\n");
        let _ = stream.write_all(response.as_bytes());
    }

    fn format_jobs_table(&self) -> String {
        let mut output = String::from("ID\tSTATUS\tCMD\t\t\t\t\tTime\tAge\n");

        for job in self.repo.list_jobs() {
            let id = job.id();
            let cmd = abbreviate_command(&job.command, 40);
            let status = job.status_label();
            let elapsed = format!("{}s", job.age_seconds());
            let runtime = format!("{}s", job.runtime_seconds());

            output.push_str(&format!("{:<3} {:<8} {:<40} {:<8} {}\n", id, status, cmd, runtime, elapsed));
        }

        output
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

fn abbreviate_command(command: &str, max_width: usize) -> String {
    let mut chars = command.chars();
    let collected: String = chars.by_ref().take(max_width).collect();
    if chars.next().is_some() {
        let prefix: String = command.chars().take(max_width.saturating_sub(3)).collect();
        format!("{prefix}...")
    } else {
        collected
    }
}
