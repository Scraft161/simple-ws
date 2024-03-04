use std::{error::Error, fs, str::FromStr};

use html_node::{
	text,
	typed::{element, elements::*, html},
	unsafe_text,
};

use lazy_static::lazy_static;

use regex::Regex;

use markdown::mdast;

use ftags::FTag;

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
			},
		},
	)?;

	Ok(format!(
		"<!DOCTYPE html>
<html>
    <head>
        <meta charset=\"utf-8\">
    </head>
    <body>
        {}
    </body>
</html>",
		html.replace("<img", "<img loading=\"lazy\"")
	))
}

/// It is time for some good ol' fuckshit.
/// Since MDAST doesn't support ids attached to headings for some dumbass reason we now have to do
/// this crap ourselves.
///
/// I sincerely apologize for whatever has made it's way into this function and the related ones;
/// but this was the only way that made sense to me at the time.
/// whoever you are; know that this might be absolute hell and that there will be noone to help
/// you.
///
/// - Scraft161
pub fn convert_wiki(path: &str) -> Result<html_node::Node, Box<dyn Error>> {
	let md = fs::read_to_string(path)?;

	let mdast = markdown::to_mdast(
		&md,
		&markdown::ParseOptions {
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
	)?;

	if let Some(doc) = traverse_mdast(mdast, false) {
		if let html_node::Node::Fragment(ref fragment) = doc {
			//dbg!(fragment);
			//let index = generate_index(fragment.clone());

			for child in &fragment.children {
				if let html_node::Node::Element(element) = child {
					if element.name == "div" {
						break;
					}
				}
			}
			//attach_index(fragment.children[i]);
			//println!("{index}");
		}
		//attach_index()

		Ok(doc)
	} else {
		Err("help".into())
	}
}

fn traverse_mdast(node: markdown::mdast::Node, ignore_p: bool) -> Option<html_node::Node> {
	match node {
		mdast::Node::Root(root) => {
			let mut children = Vec::new();
			let mut page_meta = PageMeta::new();
			for md_child in root.children {
				match traverse_mdast(md_child.clone(), false) {
					Some(child) => children.push(child),
					None => match md_child {
						mdast::Node::Yaml(yaml) => page_meta = PageMeta::from_yaml(&yaml.value),
						mdast::Node::Toml(_toml) => {
							todo!()
						}
						_ => continue,
					},
				};
			}

			// Generate Index
			let index = generate_index(html_node::Fragment {
				children: children.clone(),
			});
			// Find index of `Profile` child
			//if let Some(pos) = children.iter().position(|&x| x ==)
			let mut profile_found = false;
			let mut index_attached = false;
			for (i, item) in children.iter().enumerate() {
				//dbg!(&item);
				if profile_found {
					//dbg!(&item);
					if let html_node::Node::UnsafeText(text) = item {
						if text.text == "</div>" {
							// Our item
							children.insert(
								i + 1,
								html!(
									<div id="index" class="index">
										{index.clone()}
										{
											if let Some(tags) = page_meta.tags {
												html!(
													<div id="tags">
														<h4>In categories:</h4>
														{
															tags.iter().zip(1..).map(|(tag, _)| html!(
																<a href={format!("{tag}.tag")}>
																	{text!("{tag}")}
																</a>
															))
														}
													</div>
												)
											} else {
												html!(<>)
											}
										}
									</div>
								),
							);
							index_attached = true;
							break;
						}
					}
				} else if let html_node::Node::UnsafeText(text) = item {
					if text.text == "<div class=\"profile\">" {
						profile_found = true;
					}
				}
			}

			// If we didn't attach the index yet; just put it at the top.
			if !index_attached {
				children.insert(
					1,
					html!(
						<div id="index" class="index">
							{index}
						</div>
					),
				)
			}

			//dbg!(&children);

			let html = html!(
				<!DOCTYPE html>
				<html lang="en">
					<head>
						{
							if let Some(title) = page_meta.title {
								html!(
									<title>
										{text!("{}", title)}
									</title>
								)
							} else {
								html!(<>)
							}
						}
						<meta charset="utf-8">
						<link rel="stylesheet" href="/assets/css/wiki/master.css">
						<link rel="icon" href="/favicon.ico" sizes="any">
						<link rel="icon" href="/favicon.svg" type="image/svg+xml">
					</head>
					<body>
						<div class="pre-load" style="background-color:#1a1b26;width:100%;height:100%;position:absolute;top:0;left:0;"></div>
						{children}
					</body>
				</html>
			);

			// Set up root doc node
			Some(html)
		}
		mdast::Node::Heading(heading) => {
			let mut children = Vec::new();
			for child in heading.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child)
				}
			}

			let html_node::Node::Text(text) = &children[0] else {
				return None;
			};
			let mut title = String::from(&text.text);

			let (id, pos) = id_from_text(&text.text);
			if let Some(pos) = pos {
				title.truncate(pos);
			}

			Some(html_node::Node::Element(html_node::Element {
				name: format!("h{}", heading.depth),
				attributes: vec![(String::from("id"), Some(id))],
				children: Some(vec![html!({ text!("{title}") })]),
			}))
		}
		mdast::Node::Text(text) => {
			if text.value.starts_with(":>! ") {
				Some(render_spoiler(&text.value))
			} else if INLINE_SPOILER.is_match(&text.value) {
				Some(render_inline_spoiler(&text.value))
			} else {
				Some(text!("{}", text.value))
			}
		}
		mdast::Node::Paragraph(paragraph) => {
			let mut children = Vec::new();
			for child in paragraph.children {
				match traverse_mdast(child, ignore_p) {
					Some(child) => children.push(child),
					None => (),
				};
			}

			match ignore_p {
				true => Some(html_node::Node::Fragment(html_node::Fragment { children })),
				false => Some(html_node::Node::Element(html_node::Element {
					name: format!("p"),
					attributes: vec![],
					children: Some(children),
				})),
			}
		}
		mdast::Node::Html(html) => Some(html!({ unsafe_text!("{}", html.value) })),
		mdast::Node::Code(code) => match code.lang {
			Some(lang) => Some(html!(
				<div class="code-block">
					<span class="lang">{text!("{lang}")}</span>
					<pre>
						<code>{text!("{}", code.value)}</code>
					</pre>
				</div>
			)),
			None => Some(html!(
				<div class="code-block">
					<pre>
						<code>{text!("{}", code.value)}</code>
					</pre>
				</div>
			)),
		},
		mdast::Node::InlineCode(code) => {
			Some(html!(<code class="inline-code">{text!("{}", code.value)}</code>))
		}
		mdast::Node::Link(link) => {
			let mut children = Vec::new();
			for child in link.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child)
				}
			}

			Some(html!(
				<a href=link.url>
					{children}
				</a>
			))

			//Some(html_node::Node::Element(
			//    html_node::Element {
			//        name: format!("a"),
			//        attributes: vec![
			//            (format!("href"), Some(format!("{}", link.url)))
			//        ],
			//        children: Some(children),
			//    }
			//))
		}
		mdast::Node::Yaml(_yaml) => None,
		mdast::Node::Image(image) => {
			//dbg!(&image);
			let title = match image.title {
				Some(data) => data,
				None => "".to_string(),
			};
			let img_alt = image.alt.clone();

			if IMAGE_W_H.is_match(&image.alt) {
				let (alt, m_width, m_height) = image_props_from_text(&image.alt);

				if let (Some(width), Some(height)) = (&m_width, &m_height) {
					Some(html!(
						<img alt=alt src=image.url title=title width=format!("{width}") height=format!("{height}") loading="lazy">
					))
				} else if let Some(width) = &m_width {
					Some(html!(
						<img alt=alt src=image.url title=title width=format!("{width}") loading="lazy">
					))
				} else if let Some(height) = &m_height {
					Some(html!(
						<img alt=alt src=image.url title=title height=format!("{height}") loading="lazy">
					))
				} else {
					Some(html!(
						<img alt=img_alt src=image.url title=title loading="lazy">
					))
				}
			} else {
				Some(html!(
					<img alt=img_alt src=image.url title=title loading="lazy">
				))
			}
		}
		mdast::Node::Strong(strong) => {
			let mut children = Vec::new();
			for child in strong.children {
				match traverse_mdast(child, false) {
					Some(child) => children.push(child),
					None => (),
				};
			}

			Some(html_node::Node::Element(html_node::Element {
				name: format!("strong"),
				attributes: Vec::new(),
				children: Some(children),
			}))
		}
		mdast::Node::Emphasis(em) => {
			let mut children = Vec::new();
			for child in em.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child);
				}
			}

			Some(html_node::Node::Element(html_node::Element {
				name: String::from("em"),
				attributes: Vec::new(),
				children: Some(children),
			}))
		}
		mdast::Node::Delete(del) => {
			let mut children = Vec::new();
			for child in del.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child);
				}
			}

			Some(html_node::Node::Element(html_node::Element {
				name: String::from("s"),
				attributes: Vec::new(),
				children: Some(children),
			}))
		}
		mdast::Node::List(list) => {
			let mut children = Vec::new();
			for child in list.children {
				match traverse_mdast(child, false) {
					Some(child) => children.push(child),
					None => (),
				};
			}

			let mut attrs = Vec::new();
			if let Some(start) = list.start {
				attrs.push((String::from("start"), Some(format!("{}", start))))
			}

			Some(html_node::Node::Element(html_node::Element {
				name: String::from(if list.ordered { "ol" } else { "ul" }),
				attributes: attrs,
				children: Some(children),
			}))
		}
		mdast::Node::ListItem(li) => {
			//dbg!(&li);
			let mut children = Vec::new();
			if let Some(checked) = li.checked {
				match checked {
					true => children.push(html!(<input type="checkbox" disabled="" checked="">)),
					false => children.push(html!(<input type="checkbox" disabled="">)),
				}
			}
			for child in li.children {
				match traverse_mdast(child, !li.spread) {
					Some(child) => children.push(child),
					None => (),
				};
			}

			Some(html_node::Node::Element(html_node::Element {
				name: String::from("li"),
				attributes: Vec::new(),
				children: Some(children),
			}))
		}
		mdast::Node::Table(table) => {
			//dbg!(&table);

			//let mut children: Vec<html_node::Node> = Vec::new();
			let thead = html!(
				<tr>
				{table.children[0].children().unwrap().into_iter().zip(0..).map(|(th, i)|
					if mdast::AlignKind::None != table.align[i] {
						html!(
							<th class=match table.align[i] {
								mdast::AlignKind::Left   => "align-left",
								mdast::AlignKind::Center => "align-center",
								mdast::AlignKind::Right  => "align-right",
								mdast::AlignKind::None   => "",
							}>{
								let mut children = Vec::new();
								for child in th.children().unwrap() {
									if let Some(child) = traverse_mdast(child.clone(), false) {
										children.push(child);
									}
								}

								children
							}</th>
						)
					} else {
						html!(
							<th>{
								let mut children = Vec::new();
								for child in th.children().unwrap() {
									if let Some(child) = traverse_mdast(child.clone(), false) {
										children.push(child);
									}
								}

								children
							}</th>
						)
					}
				)}
				</tr>
			);
			let tbody = html!(
				<>
				{
					let mut data = Vec::new();
					for child in &table.children[1..] {
						//dbg!(&child);
						data.push(html!(
							<tr>
							{
								child.children().unwrap().into_iter().zip(0..).map(|(tc, i)| {
									if mdast::AlignKind::None != table.align[i] {
										html!{
											<td class=match table.align[i] {
												mdast::AlignKind::Left   => "align-left",
												mdast::AlignKind::Center => "align-center",
												mdast::AlignKind::Right  => "align-right",
												mdast::AlignKind::None   => "",
											}>{
												let mut children = Vec::new();
												for child in tc.children().unwrap() {
													if let Some(child) = traverse_mdast(child.clone(), false) {
														children.push(child);
													}
												}

												children
											}</td>
										}
									} else {
										traverse_mdast(tc.clone(), true).unwrap()
									}
								})
							}
							</tr>
						));
					}
					data
				}
				</>
			);

			//dbg!(&children[0]);

			Some(html!(
				<table>
					<thead>
						{thead}
					</thead>
					<tbody>
						{tbody}
					</tbody>
				</table>
			))
		}
		mdast::Node::TableRow(tr) => {
			let mut children = Vec::new();
			for child in tr.children {
				match traverse_mdast(child, false) {
					Some(child) => children.push(child),
					None => (),
				}
			}

			Some(html!(
				<tr>{children}</tr>
			))
		}
		mdast::Node::TableCell(td) => {
			let mut children = Vec::new();
			for child in td.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child)
				}
			}

			Some(html!(
				<td>
					{children}
				</td>
			))
		}
		mdast::Node::Break(_) => Some(html!(<br>)),
		mdast::Node::ThematicBreak(_) => Some(html!(<hr>)),
		mdast::Node::FootnoteReference(fnr) => Some(html!(
			<sup>
			<a id=format!("fnref-{}", fnr.identifier) href=format!("#fndef-{}", fnr.identifier) aria-describedby="footnote-label">{
				if let Some(label) = fnr.label {
					text!("{label}")
				} else {
					text!("")
				}
			}</a>
			</sup>
		)),
		mdast::Node::FootnoteDefinition(fnd) => {
			let mut children = Vec::new();
			for child in fnd.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child)
				}
			}

			Some(html!(
				<div class="footnote" id=format!("fndef-{}", fnd.identifier)>
					<span class="footnote-pre">{
						if let Some(label) = fnd.label {
							text!("{label}.")
						} else {
							text!("")
						}
					}</span>
					<div class="footnote-content">
						{children}<a href=format!("#fnref-{}", fnd.identifier) aria-label="Back to content">"⏎"</a>
					</div>
				</div>
			))
		}
		mdast::Node::BlockQuote(bq) => {
			let mut children = Vec::new();
			for child in bq.children {
				if let Some(child) = traverse_mdast(child, false) {
					children.push(child)
				}
			}

			Some(html!(
				<blockquote>
					{children}
				</blockquote>
			))
		}
		_ => {
			dbg!(node);
			None
		}
	}
}

fn render_spoiler(text: &str) -> html_node::Node {
	let (title, content) = text[4..].split_once('\n').unwrap();
	let (content, remainder) = content.rsplit_once("!<:").unwrap();
	html!(
		<label class="spoiler-block">
			<input type="checkbox">
			<span class="spoiler-title">{text!("{title}")}</span>
			<span class="spoiler-content">
				{text!("{content}")}
			</span>
		</label>
		{if remainder == "" {
			html!(<>)
		} else {
			text!("{remainder}")
		}}
	)
}

fn render_inline_spoiler(text: &str) -> html_node::Node {
	element! {
		spoiler("spoiler") {
			custom_attr,
		}
	}

	let matches = INLINE_SPOILER.find_iter(text);
	//dbg!(&matches);
	let mut doc = Vec::new();
	let mut lastend = 0;

	for item in matches {
		let item_text = &item.as_str()[2..item.as_str().len() - 2];
		if lastend < item.start() {
			doc.push(html!({ text!("{}", &text[lastend..item.start()]) }));
		}
		doc.push(html!(<spoiler>{text!("{item_text}")}</spoiler>));
		lastend = item.end();
	}
	if text.len() > lastend {
		doc.push(html!({ text!("{}", &text[lastend..text.len()]) }));
	}

	html!({ doc })
}

lazy_static! {
	static ref HEADING_WITH_ID: Regex = Regex::new(r" \{[a-zA-Z0-9\-_]+\}").unwrap();
	static ref INLINE_SPOILER: Regex = Regex::new(r"\|\|.+\|\|").unwrap();
	static ref IMAGE_W_H: Regex = Regex::new(r" *\{[ ]*(?:w:[ ]*)?(?<width>\d+(?<width_units>px|em|%))?[;, ]*(?:h:[ ]*)?(?<height>\d+(?<height_units>px|em|%))?[ ]*\}").unwrap();
}

fn id_from_text(text: &str) -> (String, Option<usize>) {
	if HEADING_WITH_ID.is_match(text) {
		let re_match = HEADING_WITH_ID.find(text).unwrap();
		let Some((_, id)) = re_match.as_str().split_once('{') else {
			todo!()
		};
		let Some((id, _)) = id.rsplit_once('}') else {
			todo!()
		};
		(String::from(id), Some(re_match.start()))
	} else {
		(text.to_lowercase().replace(' ', "-"), None)
	}
}

fn image_props_from_text(text: &str) -> (String, Option<String>, Option<String>) {
	let caps = IMAGE_W_H.captures(text).unwrap();
	let width = caps.name("width").map_or("", |m| m.as_str());
	let height = caps.name("height").map_or("", |m| m.as_str());

	(
		text[0..text.len() - caps[0].len()].to_string(),
		if width != "" {
			Some(format!("{}", &caps["width"]))
		} else {
			None
		},
		if height != "" {
			Some(format!("{}", &caps["height"]))
		} else {
			None
		},
	)
}

struct Index {
	sub_headings: Vec<H2>,
}

struct H2 {
	name: String,
	id: String,
	sub_headings: Vec<H3>,
}

struct H3 {
	name: String,
	id: String,
	sub_headings: Vec<H4>,
}

struct H4 {
	name: String,
	id: String,
}

fn generate_index(root: html_node::Fragment) -> html_node::Node {
	let mut index = Index {
		sub_headings: Vec::new(),
	};
	let mut in_profile = false;
	for node in root.children {
		// Let's not pull headings from profile into our index.
		if let html_node::Node::UnsafeText(ref text) = node {
			if text.text == "<div class=\"profile\">" {
				in_profile = true;
			} else if in_profile && text.text == "</div>" {
				in_profile = false;
			}
		}
		if !in_profile {
			if let html_node::Node::Element(element) = node {
				match element.name.as_str() {
					"h2" => {
						let mut name = String::new();
						for child in element.children.unwrap() {
							match child {
								html_node::Node::Text(text) => {
									name = text.text;
								}
								_ => continue,
							}
						}
						let (id, _pos) = id_from_text(&name);
						index.sub_headings.push(H2 {
							name,
							id,
							sub_headings: Vec::new(),
						})
					}
					"h3" => {
						let mut name = String::new();
						for child in element.children.unwrap() {
							match child {
								html_node::Node::Text(text) => {
									name = text.text;
								}
								_ => continue,
							}
						}
						let h2_len = index.sub_headings.len();
						if h2_len == 0 {
							continue;
						}
						let (id, _pos) = id_from_text(&name);
						index.sub_headings[h2_len - 1].sub_headings.push(H3 {
							name,
							id,
							sub_headings: Vec::new(),
						})
					}
					"h4" => {
						let mut name = String::new();
						for child in element.children.unwrap() {
							match child {
								html_node::Node::Text(text) => {
									name = text.text;
								}
								_ => (),
							}
						}
						let h2_len = index.sub_headings.len();
						if h2_len == 0 {
							continue;
						}
						let h3_len = index.sub_headings[h2_len - 1].sub_headings.len();
						if h3_len == 0 {
							continue;
						}
						let (id, _pos) = id_from_text(&name);
						index.sub_headings[h2_len - 1].sub_headings[h3_len - 1]
							.sub_headings
							.push(H4 { name, id })
					}
					_ => continue,
				}
			}
		}
	}

	let doc = html!(
		<ul>
		{
			index.sub_headings.into_iter().zip(1..).map(|(heading, _)| html!(
				<li>
					<a href=format!("#{}", heading.id)>
						{text!("{}", heading.name)}
					</a>
					{
						if !heading.sub_headings.is_empty() { html!(
							<ul>
							{
								heading.sub_headings.into_iter().zip(1..).map(|(heading, _)| html!(
									<li>
										<a href=format!("#{}", heading.id)>
											{text!("{}", heading.name)}
										</a>
										{
											if !heading.sub_headings.is_empty() {html!(
												<ul>
												{
													heading.sub_headings.into_iter().zip(1..).map(|(heading, _)| html!(
														<li>
															<a href=format!("#{}", heading.id)>
																{text!("{}", heading.name)}
															</a>
														</li>
													))
												}
												</ul>
											)} else { html!(<>) }
										}
									</li>
								))
							}
							</ul>
						)} else { html!(<>) }
					}
				</li>
			))
		}
		</ul>
	);

	doc
}

#[derive(Default)]
struct PageMeta {
	title: Option<String>,
	tags: Option<Vec<FTag>>,
}

impl PageMeta {
	fn new() -> Self {
		Self {
			..Default::default()
		}
	}

	fn from_yaml(yaml: &str) -> Self {
		// Loop over the yaml string and match based on prefix
		let mut title = String::new();
		let mut tags = Vec::new();
		for str in yaml.split("\n") {
			if let Some(yaml_title) = str.strip_prefix("title: ") {
				title = yaml_title.to_string();
			} else if let Some(yaml_tags) = str.strip_prefix("tags: ") {
				yaml_tags
					.split(',')
					.for_each(|s| tags.push(FTag::from_str(s.trim()).unwrap()))
			}
		}

		Self {
			title: if !title.is_empty() { Some(title) } else { None },
			tags: if !tags.is_empty() { Some(tags) } else { None },
		}
	}
}
