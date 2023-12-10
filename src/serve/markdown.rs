use std::{
    error::Error,
    fs
};

pub fn convert(path: &str) -> Result<String, Box<dyn Error>> {
    let md = fs::read_to_string(path)?;
    
    let html = markdown::to_html_with_options(
        &md,
        &markdown::Options {
            parse: markdown::ParseOptions {
                constructs: markdown::Constructs {
                    //character_reference: true,
                    frontmatter: true,
                    html_flow: true,
                    html_text: true,
                    ..markdown::Constructs::gfm()
                },
                gfm_strikethrough_single_tilde: false,
                ..markdown::ParseOptions::gfm()
            },
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true,
                ..markdown::CompileOptions::default()
            }
        }
    )?;

    Ok(format!("<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"utf-8\">
    </head>
    <body>
        {}
    </body>
</html>", html.replace("<img", "<img loading=\"lazy\"")))
}
