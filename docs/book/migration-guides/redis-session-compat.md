# Redis Session Compatibility (Node -> Oxidite)

This guide keeps users logged in during migration.

## Compatibility goals

- Preserve cookie name and signing behavior.
- Preserve Redis key namespace and TTL policy.
- Preserve session invalidation semantics.

## Migration approach

1. Run Oxidite and Node against the same Redis session store.
2. Validate Oxidite middleware reads existing session records.
3. Keep session payload backward-compatible through transition.

## Checklist

- Same cookie attributes (`Secure`, `HttpOnly`, `SameSite`, `Path`, `Domain`)
- Same rotation/renewal logic
- Same logout and forced-revoke behavior

## Rollout

- Start with read-only session validation endpoints.
- Enable write/update only after parity checks pass.
- Keep dual read tolerance for one release cycle.
