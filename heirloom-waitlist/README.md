# Heirloom Waitlist Platform

A waitlist website for the Heirloom family history platform built with Oxidite v2.

## Overview

Heirloom is a modern, secure, and collaborative platform that empowers users to build, manage, and explore their family history with ease and precision. This waitlist website collects early adopters' emails to notify them when the platform launches.

## Features

- Responsive landing page with information about Heirloom
- Email collection form with validation
- Integration with Brevo API for email handling
- Waitlist management system
- FAQ and features sections
- Custom logo integration
- First member notification system

## Tech Stack

- **Backend**: Rust with Oxidite v2 framework
- **Frontend**: HTML, CSS, JavaScript
- **Email Service**: Brevo API
- **Database**: In-memory storage (will be migrated to PostgreSQL in production)

## Setup

1. Clone the repository
2. Install Rust and Cargo
3. Copy `.env.example` to `.env` and fill in your Brevo API credentials:

```bash
cp .env.example .env
```

4. Install dependencies:

```bash
cargo build
```

5. Run the application:

```bash
cargo run
```

The server will start on `http://localhost:8080`.

## Environment Variables

- `BREVO_API_KEY`: Your Brevo API key
- `BREVO_LIST_ID`: The ID of the contact list in Brevo (defaults to 1)
- `SENDER_EMAIL`: The sender email address for confirmation emails
- `DATABASE_URL`: PostgreSQL database URL (for future use)

## API Endpoints

- `GET /` - Landing page
- `GET /about` - About page
- `GET /features` - Features page
- `GET /faq` - FAQ page
- `POST /api/waitlist` - Join waitlist endpoint

## Deployment

For production deployment, you'll need to:

1. Set up environment variables on your hosting platform
2. Configure a PostgreSQL database
3. Set up a reverse proxy (Nginx/Apache)
4. Configure SSL certificate
5. Set up a process manager (PM2, systemd, etc.)

## About Heirloom

Heirloom connects generations by providing a unified platform where your family's entire legacy can be preserved, shared, and celebrated for generations to come. The platform features:

- Intuitive Family Tree Builder
- Digital Vault for photos, documents, and stories
- Collaborative Workspace for family members
- Powerful Search & Discovery
- Robust data privacy and security

## License

MIT