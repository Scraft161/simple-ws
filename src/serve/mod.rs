use std::fs;

//use html_node::Node;
use snowboard::{
    headers,
    Headers,
    Request,
};

use crate::{
    WORKING_DIR,
    format_error,
};

mod markdown;

pub fn serve_file(http_request: Request, pretty: bool) -> (u16, Headers, String) {
    let mut status = 200;
    let mut mime_type = "text/html";
    let content = match http_request.url {
        _ if http_request.url.ends_with('/') => match serve_md(&(http_request.url.clone() + "index.md")) {
            Ok(data) => Ok(data),
            Err(_) => serve_html(&(http_request.url.clone() + "index.html")),
        },
        _ if http_request.url.ends_with(".md") => serve_md(&http_request.url),
        _ if http_request.url.ends_with(".html") => serve_html(&http_request.url),
        _ if http_request.url.ends_with(".css") => {
            mime_type = "text/css";
            serve_raw(&http_request.url)
        },
        _ if http_request.url.ends_with(".js") => {
            mime_type = "text/js";
            serve_raw(&http_request.url)
        },
        _ if http_request.url.ends_with(".png") => {
            mime_type = "image/png";
            serve_raw(&http_request.url)
        },
        _ => match serve_md(&(http_request.url.clone() + ".md")) {
            Ok(data) => Ok(data),
            Err(_) => serve_html(&(http_request.url.clone() + ".html")),
        }
    };

    let content = match content {
        Ok(data) => data,
        Err(_) => {
            status = 404;
            mime_type = "text/html";
            format_error(404, "Not found", "The page you are looking for has not been found.", pretty)
        }
    };

    let headers = headers! {
        "Content-Type" => mime_type,
    };

    (status, headers, content)
}

fn serve_md(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = WORKING_DIR.to_string() + path;

    markdown::convert(&path)
}

fn serve_html(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = WORKING_DIR.to_string() + path;
    //println!("{path}");
    Ok(fs::read_to_string(path)?)
}

fn serve_raw(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = WORKING_DIR.to_string() + path;

    Ok(fs::read_to_string(path)?)
}

#[derive(Debug)]
struct NotFoundError;

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "File not found!")
    }
}

impl std::error::Error for NotFoundError {}
