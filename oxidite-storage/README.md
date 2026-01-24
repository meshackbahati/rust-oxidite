# oxidite-storage

File storage for Oxidite with local and S3 backends.

## Installation

```toml
[dependencies]
oxidite-storage = "0.1"
```

## Usage

```rust
use oxidite_storage::*;

// Local storage
let storage = LocalStorage::new("uploads").unwrap();

// Store file
storage.put("file.txt", data).await?;

// Get file
let data = storage.get("file.txt").await?;

// Delete
storage.delete("file.txt").await?;
```

## License

MIT
