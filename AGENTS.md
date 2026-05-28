- Oxidite CLI v2.3.1 - Bug Report

**Tested on**: May 26, 2026
**Framework version**: 2.3.1
**CLI version**: 2.3.0 (should be updated to 2.3.1)
**Test project**: Codebana backend (PostgreSQL, binary crate with `oxidite` dependency)

---

## BUG #1: `oxidite migrate` - SQL Parser Fails on Valid PostgreSQL

**Severity**: CRITICAL  
**Command**: `oxidite migrate`  
**Status**: Fails with `syntax error at or near ","`

### Reproduction

1. Create a migration file with valid PostgreSQL SQL:
```sql
-- migrations/001_create_tables.sql
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);
```

2. Run `oxidite migrate`

**Expected**: Migration applies successfully  
**Actual**: `error returned from database: syntax error at or near ","`

### Root Cause

The `split_sql_statements()` function in `oxidite-cli/src/commands/sql_script.rs` is corrupting or incorrectly parsing the SQL. The same SQL file works perfectly when run directly with `psql -f migrations/001_create_tables.sql`.

### Verification

```bash
# This works:
psql "$DATABASE_URL" -f migrations/001_create_tables.sql
# → CREATE TABLE

# This fails:
oxidite migrate
# → syntax error at or near ","
```

### Impact

Users cannot run migrations through the CLI. They must manually run SQL with psql, defeating the purpose of having a migration system.

---

## BUG #2: `oxidite doctor` - Doesn't Load `.env` File

**Severity**: HIGH  
**Command**: `oxidite doctor`  
**Status**: Shows false negatives for environment variables

### Reproduction

1. Create `.env` file with `DATABASE_URL=postgresql://...`
2. Run `oxidite doctor`

**Expected**: Shows `DATABASE_URL: Set`  
**Actual**: Shows `DATABASE_URL: Not set`

### Output
```
Checking environment variables:
  DATABASE_URL: ⚠️  Not set
  REDIS_URL: ⚠️  Not set
  JWT_SECRET: ⚠️  Not set
```

### Root Cause

The `doctor` command doesn't call `dotenv::dotenv()` before checking environment variables. Other commands like `dev` and `migrate` do load `.env`, but `doctor` skips it.

### Location

`oxidite-cli/src/commands/doctor.rs` - missing `load_dotenv()` call at the start.

---

## BUG #3: `oxidite doctor` - Fails to Detect Project Outside Cargo Directory

**Severity**: MEDIUM  
**Command**: `oxidite doctor`  
**Status**: Shows "Not in a Cargo project directory" when run from parent directories

### Reproduction

1. Run `oxidite doctor` from `/home/user` (not in a project)
2. Shows `⚠️ Not in a Cargo project directory`

**Expected**: Should search parent directories for `Cargo.toml`  
**Actual**: Only checks current directory

### Impact

Poor DX - users expect CLI to work from anywhere within their project tree.

---

## BUG #4: `oxidite tinker` - Fails on Binary Crates

**Severity**: HIGH  
**Command**: `oxidite tinker`  
**Status**: Compilation error on binary-only crates

### Reproduction

1. Create a binary-only Rust project (no `src/lib.rs`)
2. Depend on `oxidite` with `features = ["full"]`
3. Run `oxidite tinker`
4. Type any expression like `1 + 2`

**Expected**: Evaluates `3`  
**Actual**: Compilation error:
```
error[E0432]: unresolved import `oxidite_config`
error[E0432]: unresolved import `oxidite_db`
```

### Root Cause

The tinker generates this code for binary crates:
```rust
use oxidite::prelude::*;
use oxidite_config::Config;  // ← NOT A DEPENDENCY
use oxidite_db::DbPool;      // ← NOT A DEPENDENCY
```

But the user's `Cargo.toml` only has:
```toml
[dependencies]
oxidite = { version = "2.3.1", features = ["full"] }
```

The sub-crates `oxidite_config` and `oxidite_db` are not directly available. They're re-exported through `oxidite::*`.

### Fix

For binary crates, the generated code should use:
```rust
use oxidite::prelude::*;
use oxidite::config::Config;  // ← Re-exported
use oxidite::db::DbPool;      // ← Re-exported
```

Or better yet, don't import them unless the user explicitly uses them.

---

## BUG #5: `oxidite migrate` - SQL Files Without `-- migrate:up` Markers

**Severity**: MEDIUM  
**Command**: `oxidite migrate`  
**Status**: Behavior unclear for plain SQL files

### Issue

The migration parser in `oxidite-db/src/migrations.rs` expects:
```sql
-- migrate:up
CREATE TABLE ...

-- migrate:down
DROP TABLE ...
```

But many developers write plain SQL files without markers:
```sql
CREATE TABLE ...
```

**Current behavior**: The entire file is treated as up SQL (works), but there's no down migration for rollback.

**Recommendation**: 
- Support both formats (with and without markers)
- Warn when a migration has no down migration
- Document the expected format clearly

---

## BUG #6: `oxidite serve` - Database Pool Timeout

**Severity**: MEDIUM  
**Command**: `oxidite serve`  
**Status**: `pool timed out while waiting for an open connection`

### Reproduction

1. Run `oxidite dev` (stops it with Ctrl+C)
2. Run `oxidite serve`

**Expected**: Server starts  
**Actual**: Database pool timeout

### Possible Causes

1. Previous process didn't release connections properly
2. Connection pool settings in `oxidite.toml` are too low
3. Neon PostgreSQL pooler limits are hit

### Investigation Needed

- Check if `oxidite dev` properly drops connections on exit
- Verify pool configuration defaults
- Test with local PostgreSQL to rule out Neon-specific issues

---

## BUG #7: CLI Version Mismatch

**Severity**: LOW  
**Status**: CLI reports v2.3.0, framework is v2.3.1

### Issue

```bash
oxidite --version
# → oxidite 2.3.0
```

But the framework crates are v2.3.1. The CLI Cargo.toml still shows `version = "2.3.0"`.

### Fix

Update `oxidite-cli/Cargo.toml`:
```toml
[package]
name = "oxidite-cli"
version = "2.3.1"  # ← Update this
```

Also update all inter-crate dependencies from `version = "2.3.0"` to `version = "2.3.1"`.

---

## WHAT WORKS

These commands work correctly:

| Command | Status | Notes |
|---------|--------|-------|
| `oxidite migrate status` | ✅ Works | Shows applied/pending migrations |
| `oxidite dev` | ✅ Works | Hot reload, compiles and runs project |
| `oxidite serve` | ⚠️ Pool timeout | May be Neon-specific |
| `oxidite generate model` | ✅ Works | Creates proper model struct |
| `oxidite generate controller` | ✅ Works | Creates controller file |
| `oxidite generate middleware` | ✅ Works | Creates middleware file |
| `oxidite generate service` | ✅ Works | Creates service file |
| `oxidite generate route` | ✅ Works | Creates route file |
| `oxidite doctor` | ⚠️ Partial | Doesn't load .env |
| `oxidite tinker` | ❌ Fails | Import errors on binary crates |
| `oxidite migrate` | ❌ Fails | SQL parser bug |

---

## RECOMMENDATIONS FOR v2.3.1

### Critical (Must Fix)
1. **Fix `split_sql_statements()` parser** - The core migration system is broken
2. **Fix `oxidite tinker` imports** - Use `oxidite::*` re-exports, not sub-crate imports
3. **Add `load_dotenv()` to `oxidite doctor`** - Environment checks are useless without it

### Important
4. **Update CLI version to 2.3.1** - All crate versions should match
5. **Add parent directory search** - `doctor` and other commands should find `Cargo.toml` up the tree
6. **Add integration tests** - Test CLI commands end-to-end with real PostgreSQL

### Nice to Have
7. **Warn on missing down migrations** - When `-- migrate:down` is absent
8. **Better error messages** - "syntax error at or near ','" doesn't help users debug
9. **Verbose/debug mode** - `oxidite migrate --verbose` should show SQL being executed

---

## TEST ENVIRONMENT

- **OS**: Linux (Arch Linux 7.0.10)
- **Rust**: 1.94.1
- **Database**: Neon PostgreSQL (remote)
- **Project type**: Binary crate (no lib.rs)
- **Dependencies**: `oxidite = { version = "2.3.1", features = ["full"] }`

---

*Report generated during testing of Oxidite v2.3.1 with Codebana project*
- oxidite
