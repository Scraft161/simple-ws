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

#[derive(Default)]
pub struct HttpResponse {
    pub protocol_ver: String,
    pub status_code: usize,
    pub status_text: Option<String>,
    pub headers: Vec<HttpHeader>,
    pub content: String,
}

impl std::fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut headers = String::new();
        for header in &self.headers {
            headers.push_str(&format!("{}: {}\r\n", header.key, header.val));
        }

        if let Some(status_text) = &self.status_text {
            write!(f, "{} {} {}\r\n{}\r\n{}",
                self.protocol_ver,
                self.status_code,
                status_text,
                headers,
                self.content)
        } else {
            write!(f, "{} {} {}\r\n{}\r\n{}",
                self.protocol_ver,
                self.status_code,
                status_text_from_code(self.status_code),
                headers,
                self.content)
        }
    }
}

pub fn status_text_from_code(status_code: usize) -> String {
    match status_code {
        // 1XX: INFORMATIONAL
        100 => "CONTINUE",
        101 => "SWITCHING PROTOCOLS",
        102 => "PROCESSING",
        103 => "EARLY HINTS",

        // 2XX: SUCCESS
        200 => "OK",
        201 => "CREATED",
        202 => "ACCEPTED",
        203 => "NON-AUTHORATIVE INFORMATION",
        204 => "NO CONTENT",
        205 => "RESET CONTENT",
        206 => "PARTIAL CONTENT",
        207 => "MULTI STATUS",
        208 => "ALREADY REPORTED",
        218 => "THIS IS FINE",
        226 => "IM USED",

        // 3XX: REDIRECTION
        300 => "MULTIPLE CHOICES",
        301 => "MOVED PERMANENTLY",
        302 => "FOUND",
        303 => "SEE OTHER",
        304 => "NOT MODIFIED",
        305 => "USE PROXY",
        306 => "SWITCH PROXY",
        307 => "TEMPORARY REDIRECT",
        308 => "PERMANENT REDIRECT",

        // 4XX: CLIENT ERROR
        400 => "BAD REQUEST",
        401 => "UNAUTHORIZED",
        402 => "PAYMENT REQUIRED",
        403 => "FORBIDDEN",
        404 => "NOT FOUND",
        405 => "METHOD NOT ALLOWED",
        406 => "NOT ACCEPTABLE",
        407 => "PROXY AUTHENTICATION REQUIRED",
        408 => "REQUEST TIMEOUT",
        409 => "CONFLICT",
        410 => "GONE",
        411 => "LENGTH REQUIRED",
        412 => "PRECONDITION FAILED",
        413 => "PAYLOAD TOO LARGE",
        414 => "URI TOO LONG",
        415 => "UNSUPPORTED MEDIA TYPE",
        416 => "RANGE NOT SATISFIABLE",
        417 => "EXPECTATION FAILED",
        418 => "I'M A TEAPOT",
        419 => "PAGE EXPIRED",
        421 => "MISDIRECTED REQUEST",
        422 => "UNPROCESSABLE ENTITY",
        423 => "LOCKED",
        424 => "FAILED DEPENDENCY",
        425 => "TOO EARLY",
        426 => "UPGRADE REQUIRED",
        428 => "PRECONDITION REQUIRED",
        429 => "TOO MANY REQUESTS",
        431 => "REQUEST HEADER FIELDS TOO LARGE",
        440 => "LOGIN TIME-OUT",
        451 => "UNAVAILABLE FOR LEGAL REASONS",

        // 5XX: SERVER ERROR
        500 => "INTERNAL SERVER ERROR",
        501 => "NOT IMPLEMENTED",
        502 => "BAD GATEWAY",
        503 => "SERVICE UNAVAILABLE",
        504 => "GATEWAY TIMEOUT",
        505 => "HTTP VERSION NOT SUPPORTED",
        506 => "VARIANT ALSO NEGOTIATES",
        507 => "INSUFFICIENT STORAGE",
        508 => "LOOP DETECTED",
        509 => "BANDWIDTH LIMIT EXCEEDED",
        // TODO: Finish

        // FALLBACK
        _ => "NON-STANDARD ERROR",
    }.to_string()
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
