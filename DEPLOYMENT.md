# Tatteau Deployment Guide

This guide covers deploying the Tatteau Leptos application to Fly.io.

## Prerequisites

1. **Fly.io Account**: Sign up at https://fly.io
2. **flyctl CLI**: Install the Fly.io command-line tool
3. **Docker** (optional): Only needed for local testing

## Quick Start (Free Tier)

### 1. Install flyctl

```bash
curl -L https://fly.io/install.sh | sh
```

Add to your PATH:
```bash
export FLYCTL_INSTALL="$HOME/.fly"
export PATH="$FLYCTL_INSTALL/bin:$PATH"
```

### 2. Login to Fly.io

```bash
flyctl auth login
```

### 3. Initialize Your App

```bash
./deploy.sh init
```

This will:
- Create a new Fly.io app
- Read configuration from `fly.toml`
- Set up the app but not deploy yet

### 4. Create Persistent Volume (for SQLite)

```bash
flyctl volumes create tatteau_data --size 1
```

Then uncomment the `[mounts]` section in `fly.toml`:

```toml
[mounts]
  source = "tatteau_data"
  destination = "/app/data"
```

### 5. Deploy

```bash
./deploy.sh deploy
```

Your app will be live at: `https://your-app-name.fly.dev`

## Configuration

### fly.toml

The main configuration file. Key settings:

```toml
app = "tatteau-app"  # Change this to your desired app name
primary_region = "ord"  # Chicago - change to your preferred region
```

**Popular regions:**
- `ord` - Chicago, IL
- `iad` - Ashburn, VA
- `lax` - Los Angeles, CA
- `lhr` - London, UK
- `syd` - Sydney, Australia

### Environment Variables

Set secrets (like API keys) using:

```bash
flyctl secrets set SECRET_KEY=your-secret-value
```

View all secrets:
```bash
flyctl secrets list
```

## Deployment Script Commands

The `deploy.sh` script provides several useful commands:

### Deploy Changes
```bash
./deploy.sh deploy
```

### View Logs
```bash
./deploy.sh logs
```

### Check Status
```bash
./deploy.sh status
```

### SSH into Container
```bash
./deploy.sh ssh
```

### Scale Application
```bash
./deploy.sh scale 2  # Scale to 2 instances
```

### Database Backup
```bash
./deploy.sh db-backup  # Downloads SQLite database
```

### Database Restore
```bash
./deploy.sh db-restore backup-file.db
```

### Destroy App
```bash
./deploy.sh destroy
```

## Cost Management

### Free Tier Limits
- 3 shared-cpu-1x VMs (256MB RAM each)
- 3GB persistent volume storage
- 160GB outbound data transfer

### Current Configuration (fly.toml)
```toml
[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 256
```

This stays within the free tier!

### Scaling for Production

**Small Production** (~$13/month):
```toml
[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 1024  # Upgrade to 1GB RAM
```

**Medium Production** (~$32/month):
```toml
[[vm]]
  cpu_kind = "dedicated"
  cpus = 1
  memory_mb = 2048  # 2GB RAM
```

## Database Management

### SQLite on Fly.io

The app uses a persistent volume to store the SQLite database:

1. Volume is mounted at `/app/data`
2. Database file: `/app/data/tatteau.db`
3. Survives deployments and restarts

### Remote Database Access

You have several options to query the production database remotely:

**Option 1: SSH Console with SQLite**
```bash
# Open interactive SQLite session
flyctl ssh console -C "sqlite3 /app/data/tatteau.db"

# Run a specific query
flyctl ssh console -C "sqlite3 /app/data/tatteau.db 'SELECT COUNT(*) FROM artists;'"
```

**Option 2: Download Database Locally**
```bash
# Quick backup and download
./deploy.sh db-backup

# Then query locally
sqlite3 tatteau-backup-*.db
```

**Option 3: Execute SQL from Local Machine**
```bash
# Create a helper script
cat > query-remote.sh << 'EOF'
#!/bin/bash
flyctl ssh console -C "sqlite3 /app/data/tatteau.db \"$1\""
EOF
chmod +x query-remote.sh

# Usage:
./query-remote.sh "SELECT * FROM artists LIMIT 5;"
```

**Option 4: SSH Tunnel (Advanced)**
For GUI tools like DB Browser for SQLite:
```bash
# Forward port (note: requires app to expose sqlite port)
flyctl proxy 3306:8080
# Then configure your GUI tool to connect via localhost:3306
```

### Backup Strategy

**Automated backups** (recommended):

Create a GitHub Action to backup daily:

```yaml
# .github/workflows/backup.yml
name: Backup Database
on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM
jobs:
  backup:
    runs-on: ubuntu-latest
    steps:
      - uses: superfly/flyctl-actions@master
        with:
          version: latest
      - run: |
          flyctl ssh console -C "cat /app/tatteau.db" > backup-$(date +%Y%m%d).db
          # Upload to S3, Google Cloud, etc.
```

**Manual backup:**
```bash
./deploy.sh db-backup
```

### Migration from Development

If you have a local database with data:

1. Backup local database
2. Deploy app to Fly.io
3. Restore database:
```bash
./deploy.sh db-restore tatteau.db
```

## Monitoring

### View Logs
```bash
flyctl logs
```

### Live tail logs
```bash
flyctl logs -f
```

### Monitoring Dashboard
```bash
flyctl dashboard
```

Or visit: https://fly.io/dashboard

## Troubleshooting

### App won't start

1. Check logs:
```bash
flyctl logs
```

2. Verify health checks are passing:
```bash
flyctl checks list
```

3. SSH into container:
```bash
flyctl ssh console
```

### Database errors

1. Verify volume is mounted:
```bash
flyctl ssh console -C "ls -la /app/data"
```

2. Check database permissions:
```bash
flyctl ssh console -C "ls -la /app/tatteau.db"
```

### Out of memory

Upgrade VM size in `fly.toml`:
```toml
[[vm]]
  memory_mb = 512  # or 1024
```

Then redeploy:
```bash
flyctl deploy
```

## Custom Domains

### Add Custom Domain

1. Add domain to app:
```bash
flyctl certs create yourdomain.com
```

2. Add DNS records (from output above)

3. Verify:
```bash
flyctl certs check yourdomain.com
```

## CI/CD with GitHub Actions

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to Fly.io
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: superfly/flyctl-actions/setup-flyctl@master
      - run: flyctl deploy --remote-only
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
```

Get your token:
```bash
flyctl auth token
```

Add to GitHub Secrets as `FLY_API_TOKEN`.

## Performance Optimization

### Build Caching with cargo-chef

Our Dockerfile uses the **cargo-chef pattern** (recommended by Fly.io) for optimal build performance:

- **First build**: Slower (installs all dependencies)
- **Subsequent builds**: Much faster (Docker caches dependency layers)
- Only rebuilds when dependencies change

This is automatically configured in our Dockerfile.

### Auto-scaling Configuration

The app is configured with Fly.io's auto-scaling:

```toml
[http_service]
  auto_stop_machines = "stop"
  auto_start_machines = true
  min_machines_running = 1  # Recommended for Leptos SSR apps
```

- `min_machines_running = 1`: Keeps at least one machine running (prevents cold starts)
- `auto_start_machines = true`: Starts new machines under load
- `auto_stop_machines = "stop"`: Stops idle machines to save costs

### Multiple Regions

Deploy to multiple regions for global low-latency:
```bash
flyctl regions add lax syd lhr
flyctl scale count 3
```

## Best Practices (from Fly.io & Leptos docs)

### ✅ What We're Doing Right

1. **cargo-chef pattern**: Fast rebuilds with dependency caching
2. **Port 8080**: Standard for Fly.io applications
3. **Bind to 0.0.0.0**: Required for Fly.io networking
4. **Multi-stage builds**: Small final image size
5. **min_machines_running = 1**: Prevents cold starts for SSR apps
6. **Health checks**: Automatic via Leptos homepage

### 📚 Additional Resources

- **Fly.io Rust Guide**: https://fly.io/docs/rust/
- **Leptos Deployment**: https://book.leptos.dev/deployment/ssr.html
- **cargo-chef**: https://github.com/LukeMathWalker/cargo-chef

## Support

- **Fly.io Docs**: https://fly.io/docs
- **Fly.io Community**: https://community.fly.io
- **Leptos Docs**: https://leptos.dev

## Summary

**Initial setup:**
```bash
flyctl auth login
./deploy.sh init
flyctl volumes create tatteau_data --size 1
./deploy.sh deploy
```

**Regular deployments:**
```bash
./deploy.sh deploy
```

**Cost:** Starts at $0/month (free tier), scales as needed.
