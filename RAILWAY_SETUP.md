# Railway Deployment Setup for Tatteau

This guide covers setting up and deploying the Tatteau application on Railway with a persistent SQLite database.

## Prerequisites

1. **Railway CLI**: Install the Railway CLI
   ```bash
   npm i -g @railway/cli
   ```

2. **Railway Account**: Sign up at [railway.app](https://railway.app)

3. **Login**: Authenticate with Railway
   ```bash
   railway login
   ```

## Initial Setup

### 1. Create Railway Project

You can create a project either via the Railway dashboard or CLI:

**Option A: Dashboard (Recommended)**
- Go to [railway.app/new](https://railway.app/new)
- Click "Empty Project"
- Name your project (e.g., "tatteau")

**Option B: CLI**
```bash
railway init
```

### 2. Link Local Directory to Project

```bash
./deploy.sh init
```

This will:
- Link your local directory to the Railway project
- Provide instructions for next steps

### 3. Create Volume for SQLite Database

**Via Dashboard (Recommended):**
1. Go to your project in Railway dashboard
2. Click on your service
3. Navigate to "Settings" tab
4. Scroll to "Volumes" section
5. Click "Add Volume"
6. Configure:
   - **Volume Name**: `tatteau_data`
   - **Mount Path**: `/app/data`
   - **Size**: 1GB (can be increased later)
7. Click "Add Volume"

**Via CLI:**
```bash
railway volume add
# Follow prompts to configure volume
```

### 4. Configure Environment Variables (Optional)

If you need custom environment variables beyond those in `railway.toml`:

**Via Dashboard:**
- Project > Service > Variables tab
- Add key-value pairs

**Via CLI:**
```bash
railway variables set KEY=value
```

### 5. Deploy

```bash
./deploy.sh deploy
```

This triggers a build using your `Dockerfile` and `railway.toml` configuration.

## Database Management

### Backup Database

Download the production database to a local file:

```bash
./deploy.sh db-backup
```

This creates a timestamped backup file: `tatteau-backup-YYYYMMDD-HHMMSS.db`

### Restore Database

Upload a local database file to production:

```bash
./deploy.sh db-restore tatteau-backup-20250109-143000.db
```

⚠️ **Warning**: This overwrites the production database. You'll be prompted for confirmation.

### Sync Database for Local Development

Sync the production database to `tatteau-live.db` for use with DataGrip or other tools:

```bash
./sync-db.sh
```

This:
1. Creates a backup from Railway
2. Copies it to `tatteau-live.db`
3. Cleans up old backups (keeps last 5)

## Common Operations

### View Logs
```bash
./deploy.sh logs
```

### Check Service Status
```bash
./deploy.sh status
```

### Open App in Browser
```bash
./deploy.sh open
```

### Interactive Shell
```bash
./deploy.sh shell
```

### SSH-like Bash Session
```bash
./deploy.sh ssh
```

### Restart Service
```bash
./deploy.sh restart
```

### List Environment Variables
```bash
./deploy.sh variables
```

### List Volumes
```bash
./deploy.sh volume-list
```

## Volume Backups in Railway

Railway automatically creates backups of volumes:

### Via Dashboard:
1. Go to Project > Service > Settings
2. Navigate to "Backups" tab
3. Configure backup schedule (Daily/Weekly/Monthly)
4. View and restore backups

### Backup Features:
- **Automatic Scheduling**: Set Daily, Weekly, or Monthly backups
- **Manual Backups**: Create backups on-demand
- **Point-in-time Restore**: Restore to any backup timestamp
- **Same Project Only**: Backups can only be restored within the same project/environment

### Important Backup Limitations:
- Backups are environment-specific
- Restoring removes newer backups created after the restore point
- Restored volume appears as a new mount with timestamp name
- Original volume is unmounted but retained

## Architecture & Considerations

### SQLite on Railway

**Pros:**
- ✅ Simple setup - no separate database service needed
- ✅ Low cost - included in your service plan
- ✅ Fast for read-heavy workloads
- ✅ Ideal for small-to-medium traffic
- ✅ Volume persistence ensures data survives deployments

**Cons:**
- ❌ Single instance only (no horizontal scaling with shared DB)
- ❌ Write performance limited compared to dedicated DB
- ❌ Manual backup/restore process (Railway backups help mitigate this)
- ❌ File locking issues if multiple instances try to write

**Best Practices:**
1. **Single Instance**: Keep `numReplicas = 1` in `railway.toml`
2. **Regular Backups**: Use `./deploy.sh db-backup` before major changes
3. **Monitor Volume Size**: Check usage in Railway dashboard
4. **WAL Mode**: SQLite uses WAL (Write-Ahead Logging) for better concurrency
5. **Volume Backups**: Enable automatic backups in Railway dashboard

### Database Path

The database is stored at `/app/data/tatteau.db` as configured in:
- `Dockerfile` (ENV `DATABASE_PATH`)
- Railway volume mount point (`/app/data`)

### Build Process

Railway uses your `Dockerfile` for building:
1. Builder stage: Compiles Rust/WASM with `cargo-leptos`
2. Runtime stage: Debian-based with minimal dependencies
3. Health check: HTTP request to port 8080

### Networking

- **Internal Port**: 8080 (configured in Dockerfile)
- **External URL**: Railway provides HTTPS automatically
- **Custom Domains**: Can be added in Railway dashboard

## Troubleshooting

### Build Failures

Check build logs:
```bash
railway logs --deployment
```

Common issues:
- Cargo dependency resolution: Check `Cargo.lock`
- wasm-bindgen version: Must match between local and Railway
- Out of memory: Increase service resources in dashboard

### Database Issues

**Database not persisting:**
- Verify volume is mounted at `/app/data`
- Check volume is attached in Railway dashboard

**Backup fails:**
- Ensure Railway CLI is logged in: `railway login`
- Check volume permissions in service logs

**Connection errors:**
- Verify `DATABASE_PATH=/app/data/tatteau.db` in environment
- Check file permissions (should be owned by `tatteau` user)

### CLI Issues

**Command not found:**
```bash
npm i -g @railway/cli
```

**Not logged in:**
```bash
railway login
```

**Wrong project:**
```bash
railway link
# Select correct project
```

## Migration from Fly.io

If migrating from Fly.io:

1. **Backup Fly.io database** (if you have one):
   ```bash
   flyctl ssh console -C "cat /app/data/tatteau.db" > migration-backup.db
   ```

2. **Setup Railway** (follow Initial Setup above)

3. **Restore to Railway**:
   ```bash
   ./deploy.sh db-restore migration-backup.db
   ```

4. **Update DNS** (if using custom domain):
   - Point domain to Railway's provided URL
   - Configure custom domain in Railway dashboard

## Resources

- [Railway Documentation](https://docs.railway.com/)
- [Railway CLI Reference](https://docs.railway.com/reference/cli-api)
- [Railway Volumes](https://docs.railway.com/guides/volumes)
- [Railway Backups](https://docs.railway.com/reference/backups)
- [SQLite Best Practices](https://www.sqlite.org/bestpractice.html)
