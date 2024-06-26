# mdbutler

A simple opinionated multithreaded web server and static site generator written in rust.

---

## Why

I wanted to explore server side scripting without the need for php or node, additionally I used this as an opportunity to learn about HTTP.

## Features

| Name     | Default | Description                              |
| -------- | ------- | :--------------------------------------- |
| compile  | ✅      | Compile to static site                   |
| serve    | ✅      | Serve files over HTTP (act as webserver) |
| markdown | ✅      | Process markdown                         |
| sass     | ✅      | Process sass and scss                    |
| ftags    | ❌      | use `ftags` tag indexing (WIP)           |

## Further goals

- [ ] Try to mimic NGinX's Virtualhosts.
- [x] Auto generate the status strings from the code (this is partially there in the `serve_file` function; but doesn't actually work).

## Known bugs

1. ignores markdown parsing opts in frontmatter
2. spoiler blocks fail with multiple paragraphs
3. use <details> for spoilers instead of our own bad code...
