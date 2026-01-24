# oxidite-mail

Email sending for Oxidite with SMTP support.

## Installation

```toml
[dependencies]
oxidite-mail = "0.1"
```

## Usage

```rust
use oxidite_mail::*;

// Create transport
let transport = SmtpTransport::new("smtp.gmail.com", 587).unwrap()
    .credentials("user", "password")
    .build();

// Create mailer
let mailer = Mailer::new(transport);

mailer.send(
    Email::new()
        .to("recipient@example.com")
        .subject("Hello")
        .body("Email content")
).await?;
```

## License

MIT
