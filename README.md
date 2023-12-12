# simple-ws

A simple opinionated multithreaded web server written in rust.

---

## Why

I wanted to explore server side scripting without the need for php or node, additionally I used this as an opportunity to learn about HTTP.

## Further goals

- [ ] Try to mimic NGinX's Virtualhosts.
- [x] Auto generate the status strings from the code (this is partially there in the `serve_file` function; but doesn't actually work).
