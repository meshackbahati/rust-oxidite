# Tooling Crates

## `oxidite-cli`

CLI provides:

- project creation
- model/controller/middleware and additional generators
- migrations and seed management
- dev workflow helpers

Use this for the default developer workflow in Oxidite projects.

## `oxidite-testing`

Main APIs:

- `TestRequest`
- `TestResponse`
- `TestServer`
- `test_router`
- async test macro re-export (`tokio::test`)

Use for unit/integration tests against routers/handlers with minimal setup.
