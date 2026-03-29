# PDF Export Guide

To produce a long-form PDF handbook from this book:

1. Build HTML:

```bash
mdbook build docs/book
```

2. Convert to PDF using your preferred engine.

Example with `wkhtmltopdf`:

```bash
wkhtmltopdf doc/book/book/index.html Oxidite-Complete-Handbook.pdf
```

For full-book output quality, use print styles and combine chapters in order.

## Notes for very large manuals

- Split by sections if a single file becomes too large for your PDF engine.
- Keep image assets local in the book directory.
- Use consistent heading hierarchy so table of contents is generated correctly.
