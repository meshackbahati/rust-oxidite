# What's New in Oxidite 2.3.3

Oxidite 2.3.3 introduces **namespaced environment variables** in `oxidite.toml`, making it easier to organise related secrets and configuration without long `SCREAMING_SNAKE_CASE` key lists.

This release is fully backwards-compatible. Existing `[env]` tables and `.env` files continue to work unchanged.

---

## 1. Namespaced Environment Variables

Previously, every environment variable had to live inside a single flat `[env]` table:

```toml
# Before — everything in one place
[env]
GOOGLE_CLIENT_ID     = "abc-123"
GOOGLE_CLIENT_SECRET = "secret-xyz"
GOOGLE_REDIRECT_URI  = "http://localhost:8080/callback"
STRIPE_API_KEY       = "sk_live_..."
STRIPE_WEBHOOK_KEY   = "whsec_..."
PLATFORM_NAME        = "my-app"
```

This works, but it gets unwieldy when you have many third-party integrations. In 2.3.3 you can group related variables under their own TOML table.

### Strategy 1: Flat `[env]` (unchanged)

The original approach still works exactly as before:

```toml
[env]
GOOGLE_CLIENT_ID = "abc-123"
NAME             = "my-app"
```

Each key maps directly to an environment variable. Nothing changes here.

### Strategy 2: Namespaced Tables (new)

Create a TOML table with any name that is **not** a known config section (`app`, `server`, `database`, `cache`, `queue`, `security`). The table name becomes an uppercase prefix:

```toml
[google]
client_id     = "abc-123"     # → GOOGLE_CLIENT_ID
client_secret = "secret-xyz"  # → GOOGLE_CLIENT_SECRET

[stripe]
api_key     = "sk_live_..."   # → STRIPE_API_KEY
webhook_key = "whsec_..."     # → STRIPE_WEBHOOK_KEY

[platform]
name = "my-app"               # → PLATFORM_NAME
```

The conversion rules are straightforward:

- Table name → uppercased prefix (`google` → `GOOGLE`)
- Key inside the table → uppercased suffix (`client_id` → `CLIENT_ID`)
- Combined with an underscore: `GOOGLE_CLIENT_ID`

### Strategy 3: Nested Tables (new)

For deeply-structured credentials, tables can nest. The path is flattened with underscores:

```toml
[google.oauth]
client_id     = "abc-123"     # → GOOGLE_OAUTH_CLIENT_ID
client_secret = "secret-xyz"  # → GOOGLE_OAUTH_CLIENT_SECRET

[aws.s3]
bucket   = "my-bucket"        # → AWS_S3_BUCKET
region   = "us-east-1"        # → AWS_S3_REGION

[aws.ses]
from_email = "noreply@my.app" # → AWS_SES_FROM_EMAIL
```

You can nest as deeply as needed — every level adds another `_SEGMENT` to the variable name.

### Strategy 4: `.env` File (unchanged)

Standard `.env` files continue to work:

```env
GOOGLE_CLIENT_ID=abc-123
GOOGLE_CLIENT_SECRET=secret-xyz
```

### Mixing Strategies

You can use any combination in the same `oxidite.toml`:

```toml
[env]
DATABASE_URL = "postgres://..."
JWT_SECRET   = "super-secret"

[google]
client_id     = "abc-123"
client_secret = "secret-xyz"

[google.oauth]
redirect_uri = "http://localhost:8080/callback"

[stripe]
api_key = "sk_live_..."
```

All four variables below will be available at runtime:

```rust
use std::env;

env::var("DATABASE_URL");              // from [env]
env::var("JWT_SECRET");                // from [env]
env::var("GOOGLE_CLIENT_ID");          // from [google]
env::var("GOOGLE_CLIENT_SECRET");      // from [google]
env::var("GOOGLE_OAUTH_REDIRECT_URI"); // from [google.oauth]
env::var("STRIPE_API_KEY");            // from [stripe]
```

### Resolution Order

When the same variable is defined in multiple places, the highest-priority source wins:

| Priority | Source                          | Notes                         |
|----------|---------------------------------|-------------------------------|
| 1 (top)  | Real OS environment variables   | Set by your shell, CI, Docker |
| 2        | `.env` file (dotenv)            | Per-developer local overrides |
| 3        | `[env]` flat table in TOML      | Project defaults              |
| 4        | Namespaced tables in TOML       | Project defaults              |

This means:

- **Production**: set secrets via OS env vars or your hosting platform's secret manager. They always win.
- **Local development**: put per-developer values in `.env` (which is gitignored). They override `oxidite.toml`.
- **Project defaults**: put non-secret defaults in `oxidite.toml` so new contributors get a working setup out of the box.

### Non-String Values

Namespaced tables support non-string TOML values. They are automatically converted:

```toml
[myapp]
port     = 8080      # → MYAPP_PORT = "8080"
debug    = true      # → MYAPP_DEBUG = "true"
max_conn = 25        # → MYAPP_MAX_CONN = "25"
```

The `[env]` flat table still only accepts strings (since it maps directly to `HashMap<String, String>`).

### Reading Namespaced Config Directly

Namespaced tables are also available through `config.get()` without going through `env::var()`:

```rust
let config = Config::load()?;

// Read from namespaced table directly:
let client_id: String = config.get("google.client_id").unwrap();
let redirect: String  = config.get("google.oauth.redirect_uri").unwrap();

// Check if a key exists:
if config.has_key("stripe.api_key") {
    // ...
}
```

This is useful when you want typed access or don't need the value in the OS environment.

### What Counts as a "Namespace"?

Any root-level TOML table that is **not** one of the known config sections is treated as a namespace:

| Known sections (NOT namespaces) | Unknown tables (ARE namespaces) |
|--------------------------------|-------------------------------|
| `[app]`                        | `[google]`                    |
| `[server]`                     | `[stripe]`                    |
| `[database]`                   | `[platform]`                  |
| `[cache]`                      | `[aws]`                       |
| `[queue]`                      | `[sendgrid]`                  |
| `[security]`                   | `[myapp]`                     |
| `[env]`                        | anything else you define      |

---

## 2. Fullstack Template Improvements

The `oxidite new` template for fullstack projects now includes:

- **SVG logo** at `public/images/oxidite.svg` — used as the favicon and displayed on the welcome page
- **Favicon link** in `templates/index.html` — points to `/images/oxidite.svg`
- **Comprehensive comments** in all generated files — every `mod.rs`, `main.rs`, route file, CSS, JS, and HTML template includes inline documentation explaining what each piece does and how to extend it

The logo is a plain static file. Delete it or replace it with your own — the HTML will still work, it will just show a broken image until you add a replacement.

---

## 3. Upgrading

Update your `Cargo.toml`:

```toml
[dependencies]
oxidite = "2.3.3"
```

No code changes are required. Existing `oxidite.toml` files work without modification.

To adopt namespaced env vars, simply move related keys from `[env]` into their own table:

```diff
 [env]
 DATABASE_URL = "postgres://..."
 JWT_SECRET   = "change-me"
-GOOGLE_CLIENT_ID     = "abc"
-GOOGLE_CLIENT_SECRET = "xyz"
-GOOGLE_REDIRECT_URI  = "http://localhost:8080/callback"

+[google]
+client_id     = "abc"
+client_secret = "xyz"
+redirect_uri  = "http://localhost:8080/callback"
```

All three variables still resolve to `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, and `GOOGLE_REDIRECT_URI` at runtime.
