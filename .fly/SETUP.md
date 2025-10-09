# Fly.io Setup Guide

This guide walks you through deploying Tatteau to Fly.io for the first time.

## Quick Start (GitHub Launch)

1. **Push to GitHub** (if you haven't already):
   ```bash
   git add .
   git commit -m "Add Fly.io deployment configuration"
   git push origin main
   ```

2. **Go to Fly.io Launch**:
   - Visit: https://fly.io/launch
   - Click "Launch from GitHub"
   - Sign in with GitHub
   - Select the `tatteau` repository

3. **Configure Deployment**:
   - Fly.io will detect your Dockerfile
   - Choose app name (e.g., `tatteau-app` or customize)
   - Select region (e.g., `ord` for Chicago)
   - Review `fly.toml` settings

4. **Create Database Volume** (IMPORTANT):
   ```bash
   flyctl volumes create tatteau_data --size 1
   ```
   Or Fly.io may prompt you to create it during deployment.

5. **Deploy**:
   - Click "Deploy" in the Fly.io UI
   - Or run: `flyctl deploy --remote-only`

6. **Verify**:
   ```bash
   flyctl status
   flyctl open  # Opens app in browser
   ```

## Database Setup

### Option A: Start with Empty Database
- The app will create an empty database automatically
- Populate it later with your data ingestion tools

### Option B: Deploy with Your Local Data
```bash
# After first deployment
./deploy.sh db-restore tatteau.db
```

## Enable Continuous Deployment (Optional)

The repo includes GitHub Actions for auto-deployment on push.

**Setup:**
1. Get Fly.io API token:
   ```bash
   flyctl auth token
   ```

2. Add to GitHub Secrets:
   - Go to: https://github.com/YOUR_USERNAME/tatteau/settings/secrets/actions
   - Click "New repository secret"
   - Name: `FLY_API_TOKEN`
   - Value: (paste the token)

3. Push to `main` branch → Auto deploys! 🚀

## Cost

**Free Tier Includes:**
- 3 shared-cpu VMs (256MB RAM each)
- 3GB persistent storage
- 160GB bandwidth/month

**Current Config:** Stays within free tier ✅

## Troubleshooting

**Build fails?**
```bash
flyctl logs
```

**Volume not mounted?**
```bash
flyctl ssh console -C "ls -la /app/data"
```

**Need to rebuild?**
```bash
flyctl deploy --remote-only --no-cache
```

## Next Steps

- ✅ App is deployed!
- ✅ Database is persistent
- ✅ SSL is automatic (https://)
- 📊 Monitor: `flyctl dashboard`
- 📝 Logs: `flyctl logs -f`
- 🔄 Backup DB: `./deploy.sh db-backup`

See [DEPLOYMENT.md](../DEPLOYMENT.md) for advanced configuration.
