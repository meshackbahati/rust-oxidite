# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Released]

### Added
- **CLI**: `oxidite dev` command for hot-reloading development server.
- **CLI**: Interactive `oxidite new` command with project types (Fullstack, API, Microservice, Serverless).
- **CLI**: `make` commands for generating code:
    - `make:model`
    - `make:controller`
    - `make:middleware`
- **Templates**: Enhanced Fullstack template with pre-configured static files and templates directory.
- **Static Files**: `static_handler` factory for serving files from custom directories.
- **Static Files**: `serve_static` now serves from root by default (e.g., `/style.css` -> `public/style.css`).
- **Static Files**: Added 404 handling for static file fallback routes.

### Changed
- **Docs**: Comprehensive update to README and Guides.
- **Core**: Improved error handling in `oxidite-template`.

### Fixed
- **CLI**: Fixed workspace issues in generated projects.
- **CLI**: Updated project scaffolding to use correct types and dependencies, resolving compilation errors in new projects.
