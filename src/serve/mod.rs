use std::fs;

//use html_node::Node;
use snowboard::{headers, Headers, Request};

use crate::format_error;

use crate::convert::markdown;

pub fn serve_file(http_request: Request, working_dir: &str, pretty: bool) -> (u16, Headers, Vec<u8>) {
	let mut status = 200;
	let mut mime_type = "text/html";

	let content = match http_request.url {
		_ if http_request.url.ends_with('/') => {
			match serve_md(&(http_request.url.clone() + "index.md"), &working_dir, pretty) {
				Ok(data) => Ok(data),
				Err(_) => serve_html(&(http_request.url.clone() + "index.html"), &working_dir),
			}
		}
		_ if http_request.url.ends_with(".md") => serve_md(&http_request.url, &working_dir, pretty),
		_ if http_request.url.ends_with(".html") => serve_html(&http_request.url, &working_dir),
		_ if http_request.url.ends_with(".css") => {
			mime_type = "text/css";
			serve_raw(&http_request.url, &working_dir)
		}
		#[cfg(feature = "sass")]
		_ if http_request.url.ends_with(".scss") || http_request.url.ends_with(".sass") => {
			mime_type = "text/css";
			serve_scss(&http_request.url, &working_dir)
		}
		_ if http_request.url.ends_with(".js") => {
			mime_type = "text/javascript";
			serve_raw(&http_request.url, &working_dir)
		}
		_ if http_request.url.ends_with(".png") => {
			mime_type = "image/png";
			serve_raw(&http_request.url, &working_dir)
		}
		_ if http_request.url.ends_with(".jxl") => {
			mime_type = "image/jxl";
			serve_raw(&http_request.url, &working_dir)
		}
		_ if http_request.url.ends_with(".webp") => {
			mime_type = "image/webp";
			serve_raw(&http_request.url, &working_dir)
		}
		_ if http_request.url.ends_with(".svg") => {
			mime_type = "image/svg+xml";
			serve_raw(&http_request.url, &working_dir)
		}

		_ if http_request.url.ends_with(".ttf") => {
			mime_type = "application/x-font-ttf";
			serve_raw(&http_request.url, &working_dir)
		}
		_ if http_request.url.ends_with(".woff2") => {
			mime_type = "application/font-woff2";
			serve_raw(&http_request.url, &working_dir)
		}

		_ => match serve_md(&(http_request.url.clone() + ".md"), &working_dir, pretty) {
			Ok(data) => Ok(data),
			Err(_) => serve_html(&(http_request.url.clone() + ".html"), &working_dir),
		},
	};

	let content = match content {
		Ok(data) => data,
		Err(_) => {
			status = 404;
			mime_type = "text/html";
			format_error(
				404,
				"Not found",
				"The page you are looking for has not been found.",
				pretty,
			)
			.into()
		}
	};

	let headers = headers! {
		"Content-Type" => mime_type,
	};

	(status, headers, content)
}

#[cfg(feature = "sass")]
fn serve_scss(path: &str, working_dir: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	let path = working_dir.to_string() + path;

	match grass::from_path(&path, &grass::Options::default()) {
		Ok(css) => Ok(css.into()),
		Err(why) => {
			println!("Err: Failed to parse `{path}`: {why}");
			Err(why)
		}
	}
}

fn serve_md(path: &str, working_dir: &str, pretty: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	let css_path = if path.starts_with("/wiki/") {
		Some("/assets/scss/wiki/master.scss")
	} else if path.starts_with("/read/") {
		Some("/assets/scss/reader/master.scss")
	} else if path == "/" || path == "/index" || path == "/index.html" || path == "/index.md" {
		Some("/assets/scss/index.scss")
	} else {
		None
	};

	let path = working_dir.to_string() + path;

	let html = markdown::convert_wiki(&path, css_path)?;
	if pretty {
		Ok(html.pretty().to_string().into_bytes())
	} else {
		Ok(html.to_string().into_bytes())
	}
}

fn serve_html(path: &str, working_dir: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	let path = working_dir.to_string() + path;
	//println!("{path}");
	Ok(fs::read(path)?)
}

fn serve_raw(path: &str, working_dir: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	let path = working_dir.to_string() + path;

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
