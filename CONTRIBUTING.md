# Contributing to Oxidite

Thank you for your interest in contributing to Oxidite! This document provides guidelines and instructions for contributing.

---

## 🌟 Ways to Contribute

- **Report bugs** via GitHub Issues
- **Suggest features** and enhancements
- **Improve documentation**
- **Write code** for new features or bug fixes
- **Write tests** to improve coverage
- **Review pull requests**

---

## 🚀 Getting Started

### Prerequisites

- **Rust 1.75+** ([Install Rust](https://www.rust-lang.org/tools/install))
- **Git**
- **PostgreSQL/MySQL/SQLite** (for database tests)
- **Redis** (for cache/queue tests)

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/yourusername/oxidite.git
cd oxidite

# Build all workspace crates
cargo build

# Run tests
cargo test

# Run with examples
cargo run --example hello-world
```

---

## 📝 Development Workflow

### 1. Fork and Clone

```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/oxidite.git
cd oxidite

# Add upstream remote
git remote add upstream https://github.com/yourusername/oxidite.git
```

### 2. Create a Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name
```

### 3. Make Changes

- Write clean, idiomatic Rust code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p oxidite-core

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Check documentation
cargo doc --no-deps
```

### 5. Commit

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: <type>(<scope>): <description>

git commit -m "feat(router): add query parameter extraction"
git commit -m "fix(db): resolve connection pool deadlock"
git commit -m "docs(readme): update installation instructions"
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

### 6. Push and Create PR

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create a Pull Request on GitHub
```

---

## 🎯 Coding Guidelines

### Code Style

- Use `cargo fmt` for formatting
- Follow Rust naming conventions
- Use meaningful variable and function names
- Keep functions focused and small

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create custom error types with `thiserror`
- Provide helpful error messages

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}
```

### Documentation

- Document all public APIs with `///` doc comments
- Include examples in documentation
- Use `//!` for module-level documentation

```rust
/// Creates a new user in the database.
///
/// # Arguments
///
/// * `name` - The user's full name
/// * `email` - The user's email address
///
/// # Example
///
/// ```
/// let user = User::create("Alice", "alice@example.com").await?;
/// ```
pub async fn create(name: &str, email: &str) -> Result<User> {
    // Implementation
}
```

### Testing

- Write unit tests in the same file
- Write integration tests in `tests/` directory
- Aim for >80% code coverage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("Alice", "alice@example.com");
        assert_eq!(user.name, "Alice");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = fetch_data().await;
        assert!(result.is_ok());
    }
}
```

---

## 🏗️ Project Structure

```
oxidite/
├── crates/
│   ├── oxidite-core/        # Core HTTP functionality
│   ├── oxidite-router/      # Routing system
│   ├── oxidite-middleware/  # Middleware
│   ├── oxidite-db/          # Database layer
│   └── ...                  # Other crates
├── examples/                # Example applications
├── tests/                   # Integration tests
├── docs/                    # Documentation
├── benches/                 # Benchmarks
└── scripts/                 # Utility scripts
```

---

## 🔍 Review Process

### Pull Request Checklist

Before submitting a PR, ensure:

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Examples are provided if applicable
- [ ] CHANGELOG is updated (for significant changes)

### What to Expect

- PRs will be reviewed within 1-3 business days
- Reviewers may request changes
- Once approved, maintainers will merge
- Your contribution will be credited in the changelog

---

## 🐛 Reporting Bugs

### Before Reporting

- Check if the bug has already been reported
- Try to reproduce with the latest version
- Gather relevant information

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Create a router with...
2. Send a request to...
3. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.75]
- Oxidite version: [e.g., 0.1.0]

**Additional context**
Any other relevant information.
```

---

## 💡 Feature Requests

We welcome feature suggestions! Please:

1. Check if the feature has been requested
2. Describe the use case
3. Explain why it fits Oxidite's goals
4. Suggest an implementation approach

---

## 🎓 Learning Resources

### Rust Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)

### Related Projects
- [Tokio](https://tokio.rs/)
- [Hyper](https://hyper.rs/)
- [Tower](https://github.com/tower-rs/tower)
- [Axum](https://github.com/tokio-rs/axum)

---

## 📜 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment.

### Our Standards

- Be respectful and considerate
- Welcome newcomers
- Focus on constructive criticism
- Assume good faith

### Enforcement

Unacceptable behavior may result in temporary or permanent bans. Report issues to the maintainers.

---

## 🎁 Recognition

Contributors will be:

- Listed in the CONTRIBUTORS file
- Mentioned in release notes
- Given credit in the changelog

---

## 📧 Questions?

- **GitHub Discussions**: For general questions
- **GitHub Issues**: For bugs and features
- **Email**: maintainer@oxidite.dev (for private concerns)

---

Thank you for contributing to Oxidite! 🦀❤️
