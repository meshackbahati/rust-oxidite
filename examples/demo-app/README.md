# Oxidite Demo Application

This demo application showcases all features of the Oxidite web framework.

## Running the Demo

```bash
cd examples/demo-app
cargo run
```

Visit `http://localhost:8080` to see the demo.

## Features Demonstrated

### 1. RESTful API
- `GET /api/v1/users` - List users
- `POST /api/v1/users` - Create user

### 2. Authentication
- `POST /auth/register` - User registration
- `POST /auth/login` - JWT authentication
- `POST /auth/oauth/google` - OAuth2 flow

### 3. Real-time
- WebSocket chat at `/ws`
- Server-Sent Events at `/sse`

## API Examples

### List Users
```bash
curl http://localhost:8080/api/v1/users
```

### Register User
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret","name":"John"}'
```

### Login
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"secret"}'
```

## Code Structure

```
src/
├── main.rs           # Application entry point
├── routes/
│   ├── mod.rs        # Route definitions
│   ├── api.rs        # API endpoints
│   ├── auth.rs       # Authentication
│   └── realtime.rs   # WebSocket & SSE
├── models.rs         # Data models
└── services.rs       # Business logic
```

## Learning Resources

- [Getting Started Guide](../../docs/guides/getting-started.md)
- [Database Guide](../../docs/guides/database.md)
- [Authentication Guide](../../docs/guides/authentication.md)
- [Templating Guide](../../docs/guides/templating.md)
- [Realtime Guide](../../docs/guides/realtime.md)
