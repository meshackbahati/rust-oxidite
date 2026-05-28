# Introduction to Oxidite

<p align="center">
  <img src="assets/oxidite.svg" alt="Oxidite Logo" width="180">
</p>

Welcome to the Oxidite documentation. This guide covers everything from installation to advanced features.

## What is Oxidite?

Oxidite is a batteries-included web framework for Rust. It wraps existing Rust libraries (hyper, sqlx, tokio, serde) and adds code generation to reduce boilerplate.

### What It Actually Does

- **Less boilerplate**: The `#[derive(Model)]` macro generates CRUD methods you'd write anyway
- **Convention over configuration**: Standard file structure, naming, and defaults
- **CLI tools**: Generate code, run migrations, hot-reload during development
- **Full-stack features**: ORM, auth, queues, caching, templates, websockets - all included

### What It Doesn't Do

- **It won't make Rust easy**: You still need to understand lifetimes, ownership, async, and error handling
- **It won't hide the metal**: You can drop to raw SQL, raw hyper, or raw sqlx at any time
- **It won't prevent bad code**: The framework won't save you from N+1 queries, memory leaks, or bad architecture
- **It won't compile fast**: All those derive macros and generics add to compile times

### Key Features

- **Performance**: Built on hyper and tokio for native Rust speed
- **ORM**: Convention-based with escape hatches to raw SQL when you need them
- **CLI**: Scaffolding, migrations, dev server. Saves typing, doesn't save you from thinking
- **Auth**: JWT, OAuth2, API keys, RBAC. You still need to configure them correctly
- **Queues**: Background jobs with retry logic. Redis or in-memory
- **Templates**: Jinja2-style with inheritance. Server-side rendering
- **WebSockets**: Real-time connections. You handle the message protocol

### Philosophy

"Convention over configuration" means: follow the patterns and things work smoothly. Fight the patterns and you'll write more code than if you started from scratch.

This is **not** a framework that makes decisions for you. It's a framework that gives you sensible defaults so you can focus on your actual application logic.

### Who Should Use This?

- **Rust developers** who are tired of writing the same boilerplate for every project
- **Teams** who want a standard starting point for new services
- **Anyone** who wants batteries-included but refuses to give up control

### Who Should NOT Use This?

- **Rust beginners** - Learn Rust first, then learn this framework
- **People wanting "magic"** - This is code generation, not runtime reflection
- **Projects needing ultra-fast compile times** - All those macros take time

### The Trade-offs

| You Get | You Pay |
|---------|---------|
| Less boilerplate | Longer compile times |
| Conventions | Must follow the conventions |
| Full control | Must understand the underlying libraries |
| Type safety | More upfront type definitions |
| No runtime overhead | More compile-time complexity |

## How to Use This Guide

This documentation explains what each feature does, how to use it, and where the limitations are.

Each chapter covers:
- What the feature does
- How to use it
- Where it falls short
- How to work around limitations
