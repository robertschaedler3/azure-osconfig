use std::collections::{VecDeque, HashMap};
use std::io::Read;
use std::process::Stdio;
use std::sync::{Mutex, Condvar, Arc};
use std::time::Duration;
use wait_timeout::ChildExt;

type Error = Box<dyn std::error::Error>;

pub struct Runner {
    cache: Arc<Mutex<HashMap<String, Status>>>,
    queue: Arc<Queue<Command>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            queue: Arc::new(Queue::new()),
            worker: None,
        }
    }

    // TODO: return a Result
    pub fn run(&mut self, command: Command) {
        if self.worker.is_none() {
            let queue = self.queue.clone();
            let cache = self.cache.clone();
            self.worker = Some(std::thread::spawn(move || {
                while let Some(command) = queue.pop() {
                    let id = command.id.clone();
                    let arguments = command.arguments.clone();
                    let timeout = command.timeout;
                    let single_line = command.single_line;

                    cache.lock().unwrap().insert(id.clone(), Status::Running);

                    let status = match Self::execute(arguments, timeout, single_line) {
                        Ok(status) => status,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            Status::Complete(0, "".to_string())
                        }
                    };

                    cache.lock().unwrap().insert(id, status);
                }
            }));
        }

        self.queue.push(command);
    }

    // TODO: return a Result
    pub fn cancel(&mut self, id: &str) {
        if let Some(_) = self.queue.remove(id) {
            // TODO: what about canceling while a command is running ?
            self.cache.lock().unwrap().insert(id.to_string(), Status::Canceled);
        }
    }

    fn execute(arguments: String, timeout: Option<u32>, single_line: bool) -> Result<Status, Error> {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg(arguments)
            .stdout(Stdio::piped())
            .spawn()?;

        // TODO: 0 timeout means no timeout

        let secs = Duration::from_secs(timeout.unwrap_or(0) as u64);
        match child.wait_timeout(secs)? {
            Some(status) => {
                let mut output = String::new();
                child.stdout.unwrap().read_to_string(&mut output)?;

                if single_line {
                    output = output.replace("\r", "").replace("\n", "");
                }

                Ok(Status::Complete(status.code().unwrap_or(0), output))
            },
            None => {
                child.kill()?;
                child.wait()?.code();
                Ok(Status::TimedOut)
            }
        }
    }

    pub fn status(&self, id: &str) -> Option<Status> {
        let cache = self.cache.lock().unwrap();
        cache.get(id).cloned()
    }
}

#[derive(Debug)]
pub struct Command {
    pub id: String,
    pub arguments: String,
    pub timeout: Option<u32>,
    pub single_line: bool,
}

#[derive(Clone)]
pub enum Status {
    Running,
    Complete(i32, String),
    Canceled,
    TimedOut,
}

impl Job for Command {
    fn id(&self) -> String {
        self.id.clone()
    }
}

struct Queue<T> {
    jobs: Mutex<Option<VecDeque<T>>>,
    cvar: Condvar,
}

trait Job {
    // type Status;

    fn id(&self) -> String;
    // fn status(&self) -> Self::Status;
}

impl<T> Queue<T>
where
    T: Job,
{
    fn new() -> Self {
        Self {
            jobs: Mutex::new(Some(VecDeque::new())),
            cvar: Condvar::new(),
        }
    }

    fn push(&self, job: T) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(queue) = jobs.as_mut() {
            queue.push_back(job);
            self.cvar.notify_all();
        }
    }

    fn pop(&self) -> Option<T> {
        let mut jobs = self.jobs.lock().unwrap();
        loop {
            match jobs.as_mut()?.pop_front() {
                Some(job) => return Some(job),
                None => jobs = self.cvar.wait(jobs).unwrap(),
            }
        }
    }

    fn remove(&self, id: &str) -> Option<T> {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(queue) = jobs.as_mut() {
            let pos = queue.iter().position(|job| job.id() == id);
            if let Some(pos) = pos {
                return queue.remove(pos);
            }
        }
        None
    }
}
