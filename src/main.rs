use std::{
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

use web_backend::{ThreadPool, HttpRequest, HttpResponse, HttpHeader, HttpMethod};

mod serve;

pub const WORKING_DIR: &str = "/var/www/html";
const BIND_ADDRESS: &str = "0.0.0.0";
const PORT: usize = 8080;
const THREADS: usize = 4;
const BUFFER_SIZE: usize = 1024;

fn main() {
    println!("Starting web server!");

    let listener = TcpListener::bind(format!("{BIND_ADDRESS}:{PORT}")).unwrap();
    let pool = ThreadPool::new(THREADS);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; BUFFER_SIZE];
    stream.read(&mut buffer).unwrap();

    let buffer = match String::from_utf8(buffer.to_vec()) {
        Ok(val) => val,
        Err(why) => {
            dbg!(buffer);
            println!("Could not do string conversion: {}", why);
            return;
        }
    };
    let request = HttpRequest::from(buffer);
    //dbg!(&request);
    //println!("{:#?}", request);

    if request.uri.ends_with(".png") {
        let path = WORKING_DIR.to_string() + &request.uri;
        let mut image = fs::read(path).unwrap();

        let response = HttpResponse {
            protocol_ver: String::from("HTTP/1.1"),
            status_code: 200,
            headers: vec![
                HttpHeader {
                    key: String::from("Content-Type"),
                    val: String::from("image/png"),
                },
                HttpHeader {
                    key: String::from("Content-Length"),
                    val: format!("{}", image.len()),
                },
            ],
            content: String::from(""),
            ..Default::default()
        }.to_string();

        let mut response = response.as_bytes().to_vec();
        response.append(&mut image);

        stream.write_all(&response).unwrap();
        return;
    }

    let response = if request.method == HttpMethod::Brew {
        let html = String::from("<html>\n\t<head>\n\t\t<title>Beverage not supported</title>\n\t</head>\n\t<body>\n\t\t<h1>Beverage not supported</h1>\n\t\t<p>I'm a teapot and I don't support coffee.</p>\n\t</body>\n</html>\n");
        HttpResponse {
            protocol_ver: String::from("HTTP/1.1"),
            status_code: 418,
            headers: vec![
                HttpHeader {
                    key: String::from("Content-Type"),
                    val: String::from("text/html"),
                },
                HttpHeader {
                    key: String::from("Content-Length"),
                    val: format!("{}", html.len()),
                },
            ],
            content: html,
            ..Default::default()
        }
    } else if request.uri.starts_with("/api") {
        let html = String::from("<h1>Oops!</h1><p>Payment Required.</p>");
        HttpResponse {
            protocol_ver: String::from("HTTP/1.1"),
            status_code: 402,
            headers: vec![HttpHeader {
                key: String::from("Content-Length"),
                val: format!("{}", html.len()),
            }],
            content: html,
            ..Default::default()
        }
    } else {
        serve::serve_file(request)
    };

    stream.write_all(response.to_string().as_bytes()).unwrap();
}
