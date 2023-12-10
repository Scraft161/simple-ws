use std::fs;
use web_backend::{
    HttpRequest,
    HttpResponse,
    HttpHeader,
};

use crate::WORKING_DIR;

mod markdown;

pub fn serve_file(http_request: HttpRequest) -> HttpResponse {
    let mut status_code = 200;
    let mut mime_type = "text/html";
    let mut headers = vec![];
    let html = match http_request.uri {
        _ if http_request.uri.ends_with('/') => match serve_md(&(http_request.uri.clone() + "index.md")) {
            Ok(data) => Ok(data),
            Err(_) => serve_html(&(http_request.uri.clone() + "index.html")),
        },
        _ if http_request.uri.ends_with(".md") => serve_md(&http_request.uri),
        _ if http_request.uri.ends_with(".html") => serve_html(&http_request.uri),
        _ if http_request.uri.ends_with(".css") => {
            mime_type = "text/css";
            serve_raw(&http_request.uri)
        },
        _ if http_request.uri.ends_with(".js") => {
            mime_type = "text/js";
            serve_raw(&http_request.uri)
        }
        _ if http_request.uri.ends_with(".png") => {
            mime_type = "image/png";
            serve_raw(&http_request.uri)
        }
        //_ => {
        //    Err(NotFoundError.into())
        //},
        _ => match serve_md(&(http_request.uri.clone() + ".md")) {
            Ok(data) => Ok(data),
            Err(_) => serve_html(&(http_request.uri.clone() + ".html")),
        }
    };

    let html = match html {
        Ok(data) => data,
        Err(_) => {
            status_code = 404;
            mime_type = "text/html";
            fs::read_to_string("404.html").unwrap()
        }
    };

    headers.push(HttpHeader {
        key: String::from("Content-Type"),
        val: String::from(mime_type),
    });
    headers.push(HttpHeader {
        key: String::from("Content-Length"),
        val: format!("{}", html.len()),
    });

    // Allow the browser to preload css
    if http_request.uri.contains("/wiki") {
        headers.push(HttpHeader {
            key: String::from("Link"),
            val: String::from("/assets/css/wiki/master.css"),
        });
    }

    let status_text = match status_code {
        100 => "CONTINUE",
        101 => "SWITCHING PROTOCOLS",
        102 => "PROCESSING",
        103 => "EARLY HINTS",
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
        // Skipping 3xx
        400 => "BAD REQUEST",
        401 => "UNAUTHORIZED",
        402 => "PAYMENT REQUIRED",
        403 => "FORBIDDEN",
        404 => "NOT FOUND",
        418 => "I'M A TEAPOT",
        _ => "",
    };

    HttpResponse {
        protocol_ver: String::from("HTTP/1.1"),
        status_code,
        status_text: String::from(status_text),
        headers,
        content: html,
    }
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