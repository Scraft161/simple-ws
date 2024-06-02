use std::fs;

use clap::{Args, Parser, Subcommand};

use mdbutler::{log, log_with_context};

#[cfg(feature = "build")]
use std::path::PathBuf;

#[cfg(feature = "serve")]
use std::sync::Mutex;

#[cfg(feature = "serve")]
mod serve;

#[cfg(any(feature = "markdown", feature = "sass"))]
mod convert;

#[cfg(feature = "markdown")]
use convert::markdown;

#[cfg(feature = "sass")]
use convert::sass;

#[cfg(feature = "serve")]
use lazy_static::lazy_static;

#[cfg(feature = "serve")]
const BIND_ADDRESS: &str = "0.0.0.0";
#[cfg(feature = "serve")]
const PORT: usize = 8080;
//const THREADS: usize = 4;
const PRETTY_PRINT_DEFAULT: bool = false;

#[cfg(feature = "serve")]
lazy_static! {
	static ref WORKING_DIR: Mutex<String> = Mutex::new(String::from("/var/www/html"));
}

// Our server vars
#[cfg(feature = "serve")]
pub const ASSET_PATH: &str = concatcp!("{WORKING_DIR}/assets");
#[cfg(feature = "serve")]
pub const CSS_PATH: &str = concatcp!("{ASSET_PATH}/css");

#[cfg(feature = "serve")]
use {
	const_format::concatcp,
	snowboard::{headers, response, Method, Request, Response, Server},
};

#[cfg(any(feature = "serve", feature = "markdown"))]
use html_node::{
	text,
	typed::{elements::*, html},
};

#[derive(Debug, Parser)]
struct Cli {
	#[arg(long, short)]
	directory: Option<String>,
	#[arg(short = 'f', long, default_value_t = true)]
	/// Whether to format output
	format: bool,
	#[cfg(feature = "markdown")]
	#[arg(short, long, default_value_t = true)]
	/// Convert markdown to HTML
	markdown: bool,
	#[cfg(feature = "sass")]
	#[arg(short, long, default_value_t = true, alias = "scss")]
	/// Convert SCSS and SASS to CSS
	sass: bool,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	#[cfg(feature = "build")]
	Build(BuildArgs),
	#[cfg(feature = "serve")]
	Serve(ServeArgs),
}

#[derive(Args, Debug)]
struct BuildArgs {
	#[arg(short, long)]
	/// Amount of threads to use for pre-processing.
	threads: Option<usize>,
	#[arg(short, long)]
	/// Directory to send output files to.
	output_dir: Option<String>,
}

#[derive(Args, Debug)]
struct ServeArgs {
	#[arg(short, long, default_value_t = String::from("0.0.0.0"))]
	/// Address to bind to
	address: String,
	#[arg(short, long, default_value_t = 8080)]
	/// Port to use
	port: usize,
}

fn main() -> std::result::Result<(), std::io::Error> {
	if !cfg!(feature = "build") && !cfg!(feature = "serve") {
		println!(
			"[NOTE]: This program has been compiled without both it's `build` and `serve` features.
While it is possible to compile like this, mdbutler is admittedly pretty useless without them.
Either enable the features you want or remove the `--no-default-features` flag."
		);
	}

	let cli = Cli::parse();

	dbg!(&cli);

	match cli.command {
		#[cfg(feature = "build")]
		Commands::Build(args) => build(cli.directory, args.output_dir, args.threads)?,
		#[cfg(feature = "serve")]
		Commands::Serve(args) => {
			if let Some(dir) = cli.directory {
				let mut lock = WORKING_DIR.lock().unwrap();
				*lock = dir;
				std::mem::drop(lock);
			}
			serve(&args.address, args.port)?;
		}
	}

	Ok(())
}

#[cfg(feature = "build")]
fn build(
	path: Option<String>,
	output_dir: Option<String>,
	threads: Option<usize>,
) -> std::io::Result<()> {
	let source_dir = if let Some(path) = path {
		PathBuf::from(path)
	} else {
		std::env::current_dir()?
	};

	fs::read_dir(source_dir)?
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path())
		.for_each(|path| {
			if path.is_dir() {
				// Recurse into

				// Ignore the black hole
				if path.file_name().unwrap().as_encoded_bytes() != b"node_modules" {
					let out_dir = if let Some(dir) = &output_dir {
						Some(
							dir.to_string()
								+ &("/".to_string() + path.file_name().unwrap().to_str().unwrap()),
						)
					} else {
						None
					};
					build(Some(path.display().to_string()), out_dir, threads).unwrap();
				}
			} else if path.is_file() {
				if let Some(ext) = path.extension() {
					match ext.as_encoded_bytes() {
						#[cfg(feature = "markdown")]
						b"md" | b"markdown" => {
							println!("{:?}", path)
						}
						#[cfg(feature = "sass")]
						b"scss" | b"sass" => {
							match sass::convert_to_file(
								&path,
								PathBuf::from(
									output_dir.as_ref().unwrap().to_string()
										+ &("/".to_string()
											+ &(path
												.file_stem()
												.unwrap()
												.to_str()
												.unwrap()
												.to_string() + ".css")),
								),
							) {
								Ok(_) => (),
								Err(why) => panic!("Could not compile sass/scss: {why}"),
							}
						}
						//b"ts" => { println!("{:?}", path) },
						_ => {}
					}
				}
			}
		});

	//source_dir.read_dir().into_iter().for_each(|item| {
	//	dbg!(&item);

	//	if item.is_file() {
	//		// Convert
	//	} else if item.is_dir() {
	//		// Recurse into
	//	}
	//})

	Ok(())
}

#[cfg(feature = "serve")]
fn serve(bind_address: &str, port: usize) -> snowboard::Result {
	log("Starting web server!");

	let server = Server::new(format!("{}:{}", bind_address, port))?;

	log(format!("Listening on {}", server.pretty_addr()?));

	server.run(handle_connection)
}

#[cfg(feature = "serve")]
fn handle_connection(request: Request) -> snowboard::Response {
	//dbg!(&request);
	//println!("{:#?}", request);
	
	// Get the string data out of the mutex
	let data = WORKING_DIR.lock().unwrap();
	let working_dir = data.clone();
	// Now drop the original mutex because we no longer need it.
	std::mem::drop(data);

	let pretty = if request.headers.contains_key("Pretty") {
		match request.headers.get("Pretty") {
			Some(val) => match val.to_lowercase().as_str() {
				"true" => true,
				"false" => false,
				_ => PRETTY_PRINT_DEFAULT,
			},
			None => PRETTY_PRINT_DEFAULT,
		}
	} else {
		PRETTY_PRINT_DEFAULT
	};

	// Requests to `api.`
	if let Some(host) = request.headers.get("Host") {
		if host.starts_with("api.") {
			return response! {
				payment_required,
				r#"{"Status": "Payment required"}"#,
				headers! {"Content-Type" => "text/json"}
			};
		}
	}

	if request.url.ends_with(".png") {
		let path = working_dir + &request.url;
		let image = match fs::read(path) {
			Ok(image) => image,
			Err(why) => {
				return if why.kind() == std::io::ErrorKind::NotFound {
					response!(not_found)
				} else {
					log_with_context(&why, request);
					response!(
						internal_server_error,
						format!("<h1>500 Internal server error</h1><p>{why}</p>"),
						headers! {"Content-Type" => "text/html"}
					)
				};
			}
		};

		response!(
			ok,
			image,
			headers! {
				"Content-Type" => "image/png"
			}
		)
	} else if request.method == Method::UNKNOWN {
		response!(
			im_a_teapot,
			format_error(418, "I'm a teapot", "Method not supported", pretty),
			headers! {"Content-Type" => "text/html"}
		)
	} else if request.url.starts_with("/api") {
		response!(
			payment_required,
			format_error_with_html(
				402,
				"Payment required",
				html!(
					<p>
						Send any and all credit card details to
						<a href="mailto:scraft161@tfwno.gf?subject=Infinite%20money%20glitch&body=Cardholder%20name%3A%20%0ACard%20number%3A%20%0AExpiration%3A%20%28mm%2Fyy%29%3A%20%0ASecurity%20code%3A%20">
							Scraft161.
						</a>
					</p>
				),
				pretty
			),
			headers! {"Content-Type" => "text/html"}
		)
	} else {
		let (status, headers, doc) = serve::serve_file(request, &working_dir, pretty);

		Response {
			version: snowboard::DEFAULT_HTTP_VERSION,
			status,
			status_text: "banana",
			bytes: doc.into(),
			headers: Some(headers),
		}
	}
}

#[cfg(feature = "serve")]
fn format_error(err_code: usize, err_desc: &str, err_details: &str, pretty: bool) -> String {
	#[cfg(feature = "serve")]
	{
		format_error_with_html(
			err_code,
			err_desc,
			html!(
				<p>{text!("{err_details}")}</p>
			),
			pretty,
		)
	}

	#[cfg(not(feature = "serve"))]
	format_error_console(err_code, err_desc, err_details, pretty)
}

#[cfg(feature = "serve")]
fn format_error_with_html(
	err_code: usize,
	err_desc: &str,
	custom_html: html_node::Node,
	pretty: bool,
) -> String {
	let doc = html!(
		<!DOCTYPE html>
		<html>
			<head>
				<title>{text!("{err_code}: {err_desc}")}</title>
				<link rel="style/css" href=format!("{CSS_PATH}")>
			</head>
			<body>
				<h1>{text!("{err_code}: {err_desc}")}</h1>
				{custom_html}
			</body>
		</html>
	);

	match pretty {
		true => doc.pretty().to_string(),
		false => doc.to_string(),
	}
}

#[cfg(feature = "serve")]
#[cfg(not(feature = "serve"))]
fn format_error_console(
	err_code: usize,
	err_desc: &str,
	custom_text: &str,
	pretty: bool,
) -> String {
	todo!();
}
