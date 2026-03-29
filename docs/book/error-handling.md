# Error Handling and Diagnostics

Great DX comes from precise errors and predictable behavior.

## Error layers

Use distinct error types per layer:

- transport errors (HTTP/extractor)
- auth errors
- domain validation errors
- persistence errors
- external integration errors

## Public API error design

A practical error payload:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "email is invalid",
    "details": {"field": "email"}
  }
}
```

Guidelines:

- stable `code` values for machines
- human-readable `message`
- optional structured `details`

## Mapping typed errors to status codes

Recommended map:

- validation -> 400
- unauthenticated -> 401
- unauthorized -> 403
- not found -> 404
- conflict -> 409
- rate limited -> 429
- dependency failure -> 502/503
- internal -> 500

## Logging and tracing

- include request ID in all error logs
- include domain entity IDs where safe
- avoid leaking secrets or tokens
- log root cause once; propagate typed context upward

## Macro diagnostics

For `oxidite-macros` derive errors:

- keep model fields explicit
- use supported attribute forms
- rely on compile-time diagnostics for incorrect types/attributes

## Migration safety diagnostics

Before a migration rollout:

- run parity checks for response and error shape
- run DB constraint violation scenarios
- verify not-found and authorization edge cases
