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

pub fn serve_file(http_request: Request, pretty: bool) -> (u16, Headers, Vec<u8>) {
    let mut status = 200;
    let mut mime_type = "text/html";
    let new = http_request.headers.get("Host").unwrap().starts_with("test.");

    let content = match http_request.url {
        _ if http_request.url.ends_with('/') => match serve_md(&(http_request.url.clone() + "index.md"), pretty, new) {
            Ok(data) => Ok(data),
            Err(_) => serve_html(&(http_request.url.clone() + "index.html")),
        },
        _ if http_request.url.ends_with(".md") => serve_md(&http_request.url, pretty, new),
        _ if http_request.url.ends_with(".html") => serve_html(&http_request.url),
        _ if http_request.url.ends_with(".css") => {
            mime_type = "text/css";
            serve_raw(&http_request.url)
        },
        _ if http_request.url.ends_with(".js") => {
            mime_type = "text/javascript";
            serve_raw(&http_request.url)
        },
        _ if http_request.url.ends_with(".png") => {
            mime_type = "image/png";
            serve_raw(&http_request.url)
        },
        _ if http_request.url.ends_with(".svg") => {
            mime_type = "image/svg+xml";
            serve_raw(&http_request.url)
        }

		_ if http_request.url.ends_with(".ttf") => {
			mime_type = "application/x-font-ttf";
			serve_raw(&http_request.url)
		}
		_ if http_request.url.ends_with(".woff2") => {
			mime_type = "application/font-woff2";
			serve_raw(&http_request.url)
		}

        _ => match serve_md(&(http_request.url.clone() + ".md"), pretty, new) {
            Ok(data) => Ok(data),
            Err(_) => serve_html(&(http_request.url.clone() + ".html")),
        }
    };

    let content = match content {
        Ok(data) => data,
        Err(_) => {
            status = 404;
            mime_type = "text/html";
            format_error(404, "Not found", "The page you are looking for has not been found.", pretty).into()
        }
    };

    let headers = headers! {
        "Content-Type" => mime_type,
    };

    (status, headers, content)
}

fn serve_md(path: &str, pretty: bool, new: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = WORKING_DIR.to_string() + path;

    if new {
        let html = markdown::convert_wiki(&path)?;
        if pretty {
            Ok(html.pretty().to_string().into_bytes())
        } else {
            Ok(html.to_string().into_bytes())
        }
    } else {
        Ok(markdown::convert(&path)?.into_bytes())
    }
}

fn serve_html(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = WORKING_DIR.to_string() + path;
    //println!("{path}");
    Ok(fs::read(path)?)
}

fn serve_raw(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = WORKING_DIR.to_string() + path;

    Ok(fs::read(path)?)
}

#[derive(Debug)]
struct NotFoundError;

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "File not found!")
    }
}

impl std::error::Error for NotFoundError {}
