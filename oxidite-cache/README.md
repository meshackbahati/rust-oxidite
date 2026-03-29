# oxidite-cache

Caching backends for Oxidite with in-memory and Redis implementations.

## Installation

```toml
[dependencies]
oxidite-cache = "2.1.0"
```

## Backends

- `MemoryCache`: process-local in-memory cache with TTL support
- `RedisCache`: shared cache backed by Redis

## Basic Usage

```rust
use std::time::Duration;
use oxidite_cache::{Cache, MemoryCache};

#[tokio::main]
async fn main() -> Result<(), oxidite_cache::CacheError> {
    let cache = MemoryCache::new();

    cache.set("user:1", &"Alice", Some(Duration::from_secs(60))).await?;
    let user: Option<String> = cache.get("user:1").await?;
    assert_eq!(user.as_deref(), Some("Alice"));

    Ok(())
}
```

## Remember Pattern

```rust
use std::time::Duration;
use oxidite_cache::MemoryCache;

# #[tokio::main]
# async fn main() -> Result<(), oxidite_cache::CacheError> {
let cache = MemoryCache::new();
let value: String = cache
    .remember("expensive:key", Duration::from_secs(30), || async {
        Ok("computed".to_string())
    })
    .await?;
assert_eq!(value, "computed");
# Ok(())
# }
```

## Notes

- Cache keys must be non-empty and free of control characters.
- TTL values must be greater than zero when provided.
- This crate currently does not implement tagged cache invalidation.
