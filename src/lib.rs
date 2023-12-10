use std::{
    sync::{mpsc, Arc, Mutex},
    thread
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    //println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub uri: String,
    pub protocol_ver: String,
    pub headers: Vec<HttpHeader>,
    pub content: Option<String>,
}

impl HttpRequest {
    pub fn from(request: String) -> Self {
        let (method_str, request) = request.split_once(' ').unwrap();
        let method = match method_str {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "BREW" => HttpMethod::Brew,
            _ => HttpMethod::Unknown,
        };

        let (uri, request) = request.split_once(' ').unwrap();
        let uri = uri.to_string();

        let (protocol_ver, request) = request.split_once("\r\n").unwrap();
        let protocol_ver = protocol_ver.to_string();
        //println!("{request}");

        let mut headers = Vec::new();
        let mut request = request;
        while !request.starts_with("\r\n") {
            let (key, val);
            (key, request) = request.split_once(": ").unwrap();
            (val, request) = request.split_once("\r\n").unwrap();

            headers.push(HttpHeader {
                key: key.to_string(),
                val: val.to_string(),
            })
        }

        request = &request[2..];
        //dbg!(request);

        let content = if !request.starts_with('\0') {
            Some(String::from(request))
        } else {
            None
        };

        Self {
            method,
            uri,
            protocol_ver,
            headers,
            content
        }
    }
}

pub struct HttpResponse {
    pub protocol_ver: String,
    pub status_code: usize,
    pub status_text: String,
    pub headers: Vec<HttpHeader>,
    pub content: String,
}

impl std::fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut headers = String::new();
        for header in &self.headers {
            headers.push_str(&format!("{}: {}\r\n", header.key, header.val));
        }
        write!(f, "{} {} {}\r\n{}\r\n{}",
            self.protocol_ver,
            self.status_code,
            self.status_text,
            headers,
            self.content)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HttpMethod {
    Get,
    Post,
    Brew,
    Unknown,
}

#[derive(Debug)]
pub struct HttpHeader {
    pub key: String,
    pub val: String,
}
