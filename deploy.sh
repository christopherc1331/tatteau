#!/bin/bash
# Tatteau - Railway Deployment Script

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if railway CLI is installed
if ! command -v railway &> /dev/null; then
    print_error "railway CLI is not installed. Installing..."
    npm i -g @railway/cli
    print_info "Railway CLI installed. Please run: railway login"
    exit 1
fi

# Check if user is logged in (railway whoami returns exit code 1 if not logged in)
if ! railway whoami &> /dev/null; then
    print_warn "You are not logged into Railway"
    print_info "Please run: railway login"
    exit 1
fi

# Parse arguments
ACTION=${1:-deploy}

case $ACTION in
    init)
        print_info "Initializing Railway project..."
        print_info "This will link your local directory to a Railway project"

        # Link to Railway project
        railway link

        print_info "Project linked! Now you need to:"
        print_info "1. Add a volume in Railway dashboard: Project Settings > Volumes"
        print_info "   - Volume name: tatteau_data"
        print_info "   - Mount path: /app/data"
        print_info "   - Size: 1GB (or as needed)"
        print_info ""
        print_info "2. Deploy with: ./deploy.sh deploy"
        ;;

    deploy)
        print_info "Deploying Tatteau to Railway..."

        # Deploy using Railway CLI (triggers build from railway.toml)
        railway up

        print_info "Deployment complete!"
        print_info "View your app: railway open"
        print_info "View logs: railway logs"
        print_info "Check status: railway status"
        ;;

    logs)
        print_info "Streaming logs from Railway..."
        railway logs
        ;;

    status)
        print_info "Checking service status..."
        railway status
        ;;

    shell)
        print_info "Opening shell to Railway service..."
        railway shell
        ;;

    ssh)
        print_info "Opening SSH connection to Railway service..."
        railway run bash
        ;;

    db-backup)
        print_info "Creating database backup..."
        BACKUP_FILE="tatteau-backup-$(date +%Y%m%d-%H%M%S).db"

        # Use railway ssh to cat the database file
        railway ssh "cat /app/data/tatteau.db" > $BACKUP_FILE

        if [ -s "$BACKUP_FILE" ]; then
            print_info "Backup saved to: $BACKUP_FILE"
            SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
            print_info "Backup size: $SIZE"
        else
            print_error "Backup failed - file is empty"
            rm -f "$BACKUP_FILE"
            exit 1
        fi
        ;;

    db-restore)
        print_info "Railway database restore via FileBrowser UI"
        print_info ""
        print_info "Due to Railway CLI limitations, database restore requires using the FileBrowser template."
        print_info "Follow these steps:"
        print_info ""
        print_info "1. Open your Railway dashboard: https://railway.app/dashboard"
        print_info "2. In your project, click 'New' -> 'Template'"
        print_info "3. Search for and deploy the 'FileBrowser' template"
        print_info "4. After deployment, go to FileBrowser service settings:"
        print_info "   - Variables tab: Set USE_VOLUME_ROOT=1"
        print_info "   - Volumes tab: Remove the default volume"
        print_info "   - Volumes tab: Mount your existing 'tatteau-app-tatteau-app-volume' at /data"
        print_info "5. Redeploy FileBrowser service"
        print_info "6. Generate domain for FileBrowser service (Settings -> Networking -> Generate Domain)"
        print_info "7. Open the FileBrowser URL and upload your database:"
        print_info "   - Navigate to /data/"
        print_info "   - Upload tatteau.db (will overwrite existing)"
        print_info "8. After upload, delete the FileBrowser service"
        print_info "9. Redeploy your main tatteau-app service to use the new database"
        print_info ""
        print_info "Alternative: Use Railway volume backups in the dashboard"
        print_info ""
        ;;

    restart)
        print_info "Restarting Railway service..."
        railway redeploy
        print_info "Service redeployed"
        ;;

    open)
        print_info "Opening app in browser..."
        railway open
        ;;

    variables)
        print_info "Listing environment variables..."
        railway variables
        ;;

    volume-list)
        print_info "Listing volumes..."
        railway volume list
        ;;

    *)
        echo "Tatteau Railway Deployment Tool"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  init          Initialize/link Railway project"
        echo "  deploy        Deploy app to Railway"
        echo "  logs          Stream application logs"
        echo "  status        Check service status"
        echo "  shell         Open interactive shell (Railway shell)"
        echo "  ssh           Open SSH-like bash session"
        echo "  restart       Restart the service"
        echo "  open          Open app in browser"
        echo "  variables     List environment variables"
        echo "  volume-list   List volumes"
        echo "  db-backup     Backup SQLite database to local file"
        echo "  db-restore    Restore SQLite database from local file"
        echo ""
        echo "Examples:"
        echo "  $0 init              # First time setup"
        echo "  $0 deploy            # Deploy changes"
        echo "  $0 logs              # View logs"
        echo "  $0 db-backup         # Backup database"
        echo "  $0 db-restore file.db # Restore from backup"
        ;;
esac
