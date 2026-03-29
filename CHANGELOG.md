# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
