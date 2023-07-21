Markdown Live Preview
=====================

A desktop application built using Tauri (with a Rust backend a vanilla JS
frontend) that renders a markdown document as a HTML+CSS document in real time,
using pandoc.

The project is still in very early development, and is far from being actually
usable.

# Objectives :

- [X] creating & fetching documents
- [X] document selection page
- [X] rendering a document in the app
- [~] actually transpile the markdown document with pandoc
- [ ] watch for changes and notify the frontend
- [ ] allow rendering the previewed document to a pdf file
- [ ] implementing a differ for better performence ?
- [ ] sorting documents by folders in the document selection page
- [ ] adding support for mathjax (to render latex)
- [ ] adding support for pagedJS (to get better prints)
- [ ] implementing templates
- [ ] render other kinds of documents :
	+ [ ] slideshows
	+ [ ] norg ?