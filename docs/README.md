# Oxidite Documentation

This directory contains the complete documentation for the Oxidite web framework, including a comprehensive book with tutorials and guides.

## Documentation Structure

- `/book/` - Source files for the Oxidite book (in Markdown format)
- `/book-output/` - Generated HTML documentation (created by mdBook)

## Generating HTML Documentation

The documentation is written in Markdown format and can be converted to a static HTML website using `mdBook`.

### Prerequisites

You need to have Rust and Cargo installed. Then install mdBook:

```bash
cargo install mdbook
```

### Building the Documentation

Run the following command to build the HTML documentation:

```bash
cd docs/book
mdbook build
```

The generated HTML documentation will be available in the `book-output` directory.

Alternatively, you can use the provided script:

```bash
./docs/generate_book.sh
```

### Serving Documentation Locally

To serve the documentation locally for preview:

```bash
cd docs/book
mdbook serve --open
```

This will start a local server and open the documentation in your browser.

## Book Contents

The documentation book covers:

- Introduction to Oxidite
- Installation and Setup
- Hello World Example
- Routing
- Request Handling
- Request Extractors
- Response System
- Template Engine
- Middleware
- Database ORM
- Authentication
- Background Jobs
- Real-time Features
- API Versioning
- CLI Tools
- Testing
- Plugins
- GraphQL Integration
- Production Setup
- Performance Optimization
- Features Overview

## Contributing to Documentation

To contribute to the documentation:

1. Edit the Markdown files in the `/book/` directory
2. Preview your changes by running `mdbook serve`
3. Submit a pull request with your changes

## License

The documentation is licensed under the same license as the Oxidite framework.