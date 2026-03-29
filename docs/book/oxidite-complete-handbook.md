# Oxidite Complete Handbook: From Foundations to Production Systems

This is a full learning path to become production-ready with Oxidite.

## Course outcomes

By the end of this course, learners can:

- build REST APIs and SSR apps with Oxidite
- design maintainable route/handler/service architecture
- use Oxidite ORM and raw SQL safely
- implement auth/session/authorization patterns
- run async workers, realtime events, caching, and storage
- test, observe, and deploy production services

## Audience and prerequisites

Audience:

- Rust developers building web backends
- teams migrating from Express/FastAPI/Laravel-like stacks

Prerequisites:

- Rust ownership/borrowing basics
- async/await basics
- SQL and HTTP fundamentals

## Course format

- 14 modules
- each module includes objectives, coding labs, and checkpoints
- capstone project delivered at the end

## Module 1: Foundation

Topics:

- Oxidite architecture and crate ecosystem
- request lifecycle and middleware chain
- project setup and feature flags

Lab:

- create a new project and expose health + version endpoints

## Module 2: Routing and Handlers

Topics:

- router composition
- path/query/body extraction
- response shaping and status codes

Lab:

- CRUD routes for a small resource with validation

## Module 3: Error Handling and Diagnostics

Topics:

- typed domain errors
- HTTP status mapping
- error payload conventions
- structured logging fundamentals

Lab:

- implement a unified API error envelope and handler mapping

## Module 4: Configuration and Environments

Topics:

- config loading
- environment-specific behavior
- secrets handling

Lab:

- local/dev/staging/prod config profile setup

## Module 5: Database and ORM

Topics:

- `DbPool`, models, queries, pagination
- transactions and consistency boundaries
- relation loading and soft deletes

Lab:

- build users/posts/comments domain with list filters and pagination

## Module 6: Migrations and Schema Evolution

Topics:

- migration workflow
- backward-compatible schema changes
- rollback strategy

Lab:

- add non-breaking schema migration + backfill job

## Module 7: Authentication and Authorization

Topics:

- password hashing and JWT flows
- sessions and redis-backed sessions
- RBAC/PBAC strategy

Lab:

- secure admin/user route sets with role + ownership checks

## Module 8: Caching and Performance

Topics:

- read-through caching
- invalidation strategy
- pagination/indexing patterns

Lab:

- cache expensive list endpoint with measurable speedup

## Module 9: Jobs and Async Work

Topics:

- queue backends
- retries and dead-letter patterns
- idempotent handlers

Lab:

- email + notification job pipeline with retry policy

## Module 10: Realtime and Event-Driven Flows

Topics:

- websocket and SSE usage
- pub/sub channels and room semantics
- event contract versioning

Lab:

- realtime leaderboard/notification stream

## Module 11: File and Media Handling

Topics:

- local and S3 storage backends
- validation rules and upload security

Lab:

- secure file upload endpoint + metadata persistence

## Module 12: API Documentation and Integrations

Topics:

- OpenAPI generation
- GraphQL integration patterns
- plugin architecture basics

Lab:

- expose OpenAPI docs + one GraphQL endpoint

## Module 13: Testing Strategy

Topics:

- unit/integration/contract testing
- test fixtures and deterministic state
- migration and transaction tests

Lab:

- full test suite for one bounded domain

## Module 14: Production Deployment and Operations

Topics:

- health checks and readiness probes
- rollout strategies and rollback plans
- telemetry and incident triage

Lab:

- deploy staged release and run synthetic smoke tests

## Capstone project

Build a multi-tenant event platform with:

- auth + roles
- relational models
- queue workers
- realtime updates
- cached analytics endpoint
- OpenAPI docs
- full automated tests

## Recommended pace

- intensive: 2-3 weeks full-time
- standard: 8-12 weeks part-time

## Assessment rubric

- correctness (40%)
- code quality and architecture (25%)
- tests and observability (20%)
- performance and reliability (15%)

## Instructor/mentor checklist

- review architecture before Module 5 and Module 10
- enforce typed errors and test requirements each module
- require capstone production-readiness checklist signoff

## Certification criteria (optional)

- capstone passes all tests
- deployment checklist completed
- code review meets style and reliability gates
