# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.3-1] - 2026-06-23

### Added
- **`Error::internal()` Helper**: New method `Error::internal(err)` preserves the original source error in `InternalServerErrorWithSource` variant, retaining type info and backtrace for debugging.
- **`OxiditeResponse` Convenience Methods**: Added `status()`, `headers()`, `headers_mut()` on `OxiditeResponse` for tests and handler code that only need status/header access without body conversion.
- **`From<OxiditeResponse>` for `hyper::Response<B>`**: Generic conversion for any body type `B: Default + From<BoxBody>`, fixing integration test ergonomics.
- **CORS Builder Methods**: `CorsConfig` now has `.allow_origin()`, `.allow_method()`, `.allow_header()`, `.credentials()`, `.max_age()` chainable builders.
- **`Result<T>` Closure Helper**: `pub fn Ok<T>(value: T)` shadows `std::result::Result::Ok` to fix type inference in `.map()` closures — no more turbofish needed.
- **`json!` Macro in Prelude**: `serde_json::json!` is re-exported as `json!` in `oxidite::prelude`, so users write `Response::json(json!({...}))` without any serde import.
- **Scaffolded Test Files**: `oxidite new` now generates `tests/integration_test.rs` with a working test using `oxidite-testing`.

### Changed
- Bumped `oxidite-core`, `oxidite-testing`, `oxidite-middleware`, `oxidite-template`, `oxidite-openapi`, `oxidite-graphql`, `oxidite-plugin`, `oxidite-auth`, `oxidite`, and `oxidite-cli` to `2.3.3-1`.
- Updated dependency version references across all dependant crates.
- Book HTML documentation updated to `v2.3.3-1`.
- CLI templates now use `Response::json_val(json!(...))` instead of `serde_json::json!(...)` for better ergonomics.

### Fixed
- **`Result<T>` Type Inference (#1)**: Compilation failure in closure/iterator mapping patterns resolved — see `Ok<T>` helper above.
- **`OxiditeResponse` Conversion (#2)**: Integration tests can now convert `OxiditeResponse` into `hyper::Response<Incoming>`.
- **`CorsConfig` Builder Pattern (#5)**: Added missing chainable setters for all CORS fields.
- **`TestServer` Ergonomics (#6)**: Added `Clone` impl and expanded documentation clarifying single-threaded design.
- **`Error::InternalServerError` Source Loss (#7)**: New `InternalServerErrorWithSource` variant and `Error::internal()` preserve the original error chain.
- **`render_ignition_error` Version**: Error page footer now displays `v2.3.3-1` instead of stale `v2.2.1`.

## [2.3.2] - 2026-06-07

### Added
- **Expanded Handler Capacity**: Handlers now support up to 12 extractors (previously limited to 3), enabling complex production controllers without workarounds.
- **Global Router Layering**: Introduced `Router::layer()` and `Router::with_layer()` for applying Tower-compatible middleware globally across all routes.
- **Authenticated WebSocket Support**: Enhanced `WebSocketUpgrade` to capture and preserve request extensions (auth state, etc.) during the protocol upgrade.
- **Direct Database Access**: Added `.pool()`, `.inner()`, and `.as_sqlx_pool()` to `DbPool`, providing raw `sqlx` access for advanced queries.
- **Universal Model IDs**: The `Model` derive macro now supports `Uuid` and `String` for primary keys, in addition to `i64`.
- **Streamlined API Discoverability**: Re-exported `extract`, `middleware`, and `config` modules at the crate root of `oxidite`.
- **CORS Config Integration**: Added native CORS method and header configuration support in `oxidite.toml`.
- **Enhanced Prelude**: Included `PathParams` in the prelude for easier custom extractor implementation.

### Changed
- **ORM Error Surface**: Refactored `OrmError::NotFound` to store IDs as `String` for universal compatibility across ID types.
- **Developer Documentation**: Significantly improved `FromRequest` trait documentation with real-world implementation examples.

### Fixed
- **State Resolution**: Resolved a critical issue where global router state was sometimes inaccessible within certain handler contexts.
- **WebSocket Context Loss**: Fixed architectural bug where extracted auth context was lost during the WebSocket handshake.

## [2.2.0] - 2026-05-16

### Added
- **Native WebSocket Orchestration**: Implemented a professional-grade `WebSocketUpgrade` extractor in `oxidite-core`, featuring automated 101 Switching Protocols handshake generation and header validation.
- **Thread-Safe State Management**: Added `with_state()` to `Router` utilizing `Arc<RwLock<Extensions>>` for reliable, concurrent shared state access across handlers.
- **Automated Declarative Migrations**: Introduced a powerful schema drift detection engine in `oxidite-cli`. Users can now generate incremental SQL migrations (UP/DOWN) directly from Rust model changes via `oxidite make-migrations`.
- **Introspective Database Layer**: Added the `DbInspector` trait to `oxidite-db`, enabling runtime schema inspection and validation across PostgreSQL, MySQL, and SQLite.
- **Self-Describing Models**: Enhanced the `Model` trait and its derive macro to automatically generate schema metadata, bridging the gap between Rust type safety and database structure.
- **Enhanced Framework Prelude**: Integrated `StatusCode`, `mpsc`, and `BodyExt` re-exports directly into the `oxidite` prelude to streamline asynchronous logic and HTTP status management.
- **Protocol Foundations**: Added `sha1` and `base64` dependencies to the core kernel to support low-level protocol requirements natively.

### Changed
- **Strategic Rebranding**: Executed a comprehensive framework overhaul to project an enterprise-grade, mission-critical positioning across all documentation, metadata, and introductory materials.
- **Macro Robustness**: Refactored the `Model` derive macro in `oxidite-macros` to utilize explicit absolute paths (`::oxidite::db`), ensuring reliable crate resolution across complex workspace architectures.

### Fixed
- **Crate Resolution**: Resolved critical bugs in the procedural macro layer that prevented successful compilation in projects using the umbrella framework structure.

## [2.1.0] - 2026-03-29

### Added
- Typed ORM error surface with ergonomic query builder extensions (`ModelQuery`, pagination, sort helpers, `find_or_fail`)
- Relation eager-loading helpers and bulk operation primitives in `oxidite-db`
- Checked migration APIs with typed migration errors and backend-specific migration-table DDL
- Macro diagnostics improvements for `#[derive(Model)]` misuse cases with expanded `trybuild` coverage
- Shared SQL script execution utility in CLI commands
- CLI integration tests for real subcommands in temporary project directories
- Additional `oxidite make` generators (`job`, `policy`, `event`)
- Expanded mdBook deployment support to static HTML root (`doc/book/book`) with index/search assets
- Deep migration assessment documentation for external project interoperability (notably `g24sec`)

### Changed
- Unified workspace crate versioning to `2.1.0`
- Roadmap updated with Batch B marked complete and promoted as single planning source
- Status document simplified to avoid roadmap/status drift

## [2.0.0] - 2026-01-21

### Added
- Complete rewrite of the framework with modular architecture
- HTTP/1.1, HTTP/2, and HTTP/3 server support
- Advanced ORM with relationships, migrations, soft deletes, validation
- Authentication and authorization (JWT, OAuth2, RBAC, 2FA, API keys)
- Background job queues with PostgreSQL, Redis, and memory backends
- Caching layer with memory and Redis backends
- Real-time features with WebSocket and SSE support
- Template engine with inheritance and auto-escaping
- Email sending with SMTP support
- File storage with local and S3 backends
- Security utilities (hashing, encryption, sanitization)
- Plugin system with hooks and lifecycle management
- GraphQL integration with schema generation
- Advanced middleware (rate limiting, CORS, compression, security headers)
- Comprehensive CLI tools with hot reload functionality
- Testing framework with utilities and helpers
- OpenAPI/Swagger documentation generation
- Request/Response aliases (Request/Response as shortcuts for OxiditeRequest/OxiditeResponse)
- Enhanced cookie parsing with security validations and URL decoding
- Production-ready documentation structure with consolidated features
- README files for all subcrates (oxidite-config, oxidite-graphql, oxidite-macros, oxidite-plugin)

### Changed
- Major architectural overhaul to modular crate structure
- Updated to modern Rust async/await patterns
- Enhanced error handling with detailed HTTP status code mapping
- Improved request/response types with convenient aliases
- Production-ready configuration and deployment tools
- Enhanced documentation and examples
- Consolidated documentation to eliminate redundancy (merged advanced-features, features-added, new-features, enterprise-features, api-reference into single features.md)
- Updated all crate versions to 2.0.0 for consistency
- Improved cookie parsing implementation with security considerations
- Enhanced code comments to be more human-like and natural-sounding

### Fixed
- Various stability and performance improvements
- Security vulnerabilities addressed
- Improved error handling and debugging capabilities
- Corrected inconsistencies between documentation and implementation
- Fixed extractor exports in core module
- Resolved issues with state injection in examples

## [1.0.0] - 2024-12-07

### Added
- Initial release of Oxidite framework
- Basic routing and middleware support
- Simple ORM implementation
- Authentication features
- Template engine
