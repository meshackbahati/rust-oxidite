# Heirloom Waitlist Website Deployment Guide

This document outlines the steps to deploy the Heirloom waitlist website to a production environment.

## Prerequisites

- Domain name registered (e.g., heirloomplatform.com)
- Access to a hosting provider (AWS, DigitalOcean, Heroku, etc.)
- SSL certificate (or plan to use Let's Encrypt)
- PostgreSQL database provisioned
- Brevo account with API key configured

## Deployment Options

### Option 1: Cloud Platforms (Heroku, Render, Railway)

1. **Prepare for deployment:**
   - Ensure all environment variables are configured
   - Set up PostgreSQL database and update DATABASE_URL
   - Test locally with production settings

2. **Deploy:**
   - Connect your GitHub repository to the platform
   - Configure buildpacks (Rust/Procfile if needed)
   - Set environment variables in the platform dashboard
   - Deploy the application

3. **Configure domain:**
   - Point your domain DNS to the platform
   - Set up SSL certificate

### Option 2: VPS/Dedicated Server

1. **Server preparation:**
   ```bash
   # Update system packages
   sudo apt update && sudo apt upgrade -y
   
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Install nginx
   sudo apt install nginx -y
   
   # Install PostgreSQL
   sudo apt install postgresql postgresql-contrib -y
   ```

2. **Set up the application:**
   ```bash
   # Create app directory
   sudo mkdir -p /var/www/heirloom-waitlist
   sudo chown $USER:$USER /var/www/heirloom-waitlist
   
   # Clone or copy the application files
   cd /var/www/heirloom-waitlist
   git clone <repository-url> .
   
   # Install dependencies and build
   cargo build --release
   ```

3. **Configure environment:**
   ```bash
   # Create environment file
   cp .env.example .env
   # Edit .env with appropriate values
   nano .env
   ```

4. **Set up PostgreSQL database:**
   ```bash
   # Switch to postgres user
   sudo -u postgres psql
   
   # Create database and user
   CREATE DATABASE heirloom_waitlist;
   CREATE USER heirloom_user WITH ENCRYPTED PASSWORD 'secure_password';
   GRANT ALL PRIVILEGES ON DATABASE heirloom_waitlist TO heirloom_user;
   \q
   ```

5. **Create systemd service:**
   ```bash
   sudo nano /etc/systemd/system/heirloom-waitlist.service
   ```
   
   Add the following content:
   ```
   [Unit]
   Description=Heirloom Waitlist Service
   After=network.target
   
   [Service]
   Type=simple
   User=www-data
   WorkingDirectory=/var/www/heirloom-waitlist
   ExecStart=/var/www/heirloom-waitlist/target/release/heirloom-waitlist
   Restart=always
   EnvironmentFile=/var/www/heirloom-waitlist/.env
   
   [Install]
   WantedBy=multi-user.target
   ```

6. **Enable and start the service:**
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable heirloom-waitlist
   sudo systemctl start heirloom-waitlist
   sudo systemctl status heirloom-waitlist
   ```

7. **Configure Nginx:**
   ```bash
   sudo nano /etc/nginx/sites-available/heirloom-waitlist
   ```
   
   Add the following content:
   ```
   server {
       listen 80;
       server_name heirloomplatform.com www.heirloomplatform.com;
       
       location / {
           proxy_pass http://127.0.0.1:8080;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   ```
   
   Enable the site:
   ```bash
   sudo ln -s /etc/nginx/sites-available/heirloom-waitlist /etc/nginx/sites-enabled/
   sudo nginx -t
   sudo systemctl restart nginx
   ```

8. **Set up SSL with Let's Encrypt:**
   ```bash
   sudo apt install certbot python3-certbot-nginx -y
   sudo certbot --nginx -d heirloomplatform.com -d www.heirloomplatform.com
   ```

### Option 3: Docker Deployment

1. **Create Dockerfile:**
   ```Dockerfile
   FROM rust:1.75 as builder

   WORKDIR /app
   COPY . .

   RUN cargo build --release

   FROM debian:buster-slim
   RUN apt-get update && apt-get install -y \
       ca-certificates \
       && rm -rf /var/lib/apt/lists/*

   WORKDIR /app
   COPY --from=builder /app/target/release/heirloom-waitlist /usr/local/bin/heirloom-waitlist
   COPY --from=builder /app/public ./public
   COPY --from=builder /app/templates ./templates

   EXPOSE 8080

   CMD ["heirloom-waitlist"]
   ```

2. **Build and run container:**
   ```bash
   docker build -t heirloom-waitlist .
   docker run -d -p 8080:8080 --env-file .env heirloom-waitlist
   ```

## Post-Deployment Steps

1. **Monitor application:**
   - Check logs regularly
   - Set up monitoring and alerting
   - Monitor waitlist signups

2. **Security:**
   - Regular updates
   - Firewall configuration
   - SSL certificate renewal

3. **Backup:**
   - Database backups
   - Application configuration backups

4. **Performance:**
   - Monitor response times
   - Optimize as needed
   - Scale resources as waitlist grows

## Troubleshooting

Common issues and solutions:

- **Application not starting:** Check logs with `journalctl -u heirloom-waitlist` or check the process manually
- **SSL certificate issues:** Verify DNS records and certificate validity
- **Database connection errors:** Confirm PostgreSQL is running and credentials are correct
- Environment variables not loading: Ensure .env file is in the correct location and readable
- First member notification not working: Make sure NOTIFICATION_EMAIL is set in your environment variables
- Logo not displaying: Ensure logo.jpeg is in the public/static directory

## Scaling Considerations

As the waitlist grows:
- Consider load balancing across multiple instances
- Set up a proper database connection pool
- Implement caching for static assets
- Monitor database performance
- Plan for migration to the full Heirloom platform