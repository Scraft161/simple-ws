use std::{
    fs,
};

use snowboard::{headers, response, Response, Request, Method, Server};
use html_node::{
    text,
    typed::{
        html,       // Main macro
        elements::*, // Typed html elements
    }
};
use const_format::concatcp;

use web_backend::{
    log,
    log_with_context,
};

mod serve;

pub const WORKING_DIR: &str = "/var/www/html";
const BIND_ADDRESS: &str = "0.0.0.0";
const PORT: usize = 8080;
//const THREADS: usize = 4;
const PRETTY_PRINT_DEFAULT: bool = true;

// Our server vars
pub const ASSET_PATH: &str = concatcp!("{WORKING_DIR}/assets");
pub const CSS_PATH: &str = concatcp!("{ASSET_PATH}/css");

fn main() -> snowboard::Result {
    log("Starting web server!");

    let server = Server::new(format!("{}:{}", BIND_ADDRESS, PORT))?;

    log(format!("Listening on {}", server.pretty_addr()?));

    server.run(handle_connection)
}

fn handle_connection(request: Request) -> snowboard::Response {
    //dbg!(&request);
    //println!("{:#?}", request);

    let pretty = if request.headers.contains_key("Pretty") {
        match request.headers.get("Pretty") {
            Some(val) => {
                match val.to_lowercase().as_str() {
                    "true" => true,
                    "false" => false,
                    _ => PRETTY_PRINT_DEFAULT,
                }
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
        let path = WORKING_DIR.to_string() + &request.url;
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

        response!(ok, image, headers! {
            "Content-Type" => "image/png"
        })
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
        let (status, headers, doc) = serve::serve_file(request, pretty);

        Response {
            version: snowboard::DEFAULT_HTTP_VERSION,
            status,
            status_text: "banana",
            bytes: doc.into(),
            headers: Some(headers),
        }
    }
}

fn format_error(err_code: usize, err_desc: &str, err_details: &str, pretty: bool) -> String {
    format_error_with_html(
        err_code,
        err_desc,
        html!(
            <p>{text!("{err_details}")}</p>
        ),
        pretty
    )
}

fn format_error_with_html(err_code: usize, err_desc: &str, custom_html: html_node::Node, pretty: bool) -> String {
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
        true    => doc.pretty().to_string(),
        false   => doc.to_string(),
    }
}
