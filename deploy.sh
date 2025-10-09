#!/bin/bash
# Tatteau - Fly.io Deployment Script

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

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null; then
    print_error "flyctl is not installed. Installing..."
    curl -L https://fly.io/install.sh | sh
    print_info "Please restart your terminal or run: export FLYCTL_INSTALL=\"\$HOME/.fly\""
    print_info "Then add to PATH: export PATH=\"\$FLYCTL_INSTALL/bin:\$PATH\""
    exit 1
fi

# Check if user is logged in
if ! flyctl auth whoami &> /dev/null; then
    print_warn "You are not logged into Fly.io"
    print_info "Please run: flyctl auth login"
    exit 1
fi

# Parse arguments
ACTION=${1:-deploy}

case $ACTION in
    init)
        print_info "Initializing Fly.io app..."
        print_info "This will create a new app and configure it"

        # Launch the app (this will read fly.toml)
        flyctl launch --no-deploy

        print_info "App initialized! Now create a volume for the database:"
        print_info "  flyctl volumes create tatteau_data --size 1"
        print_info ""
        print_info "Then uncomment the [mounts] section in fly.toml"
        print_info "Finally, deploy with: ./deploy.sh deploy"
        ;;

    deploy)
        print_info "Deploying Tatteau to Fly.io..."

        # Deploy the app
        flyctl deploy

        print_info "Deployment complete!"
        print_info "View your app: flyctl open"
        print_info "View logs: flyctl logs"
        print_info "Check status: flyctl status"
        ;;

    logs)
        print_info "Streaming logs from Fly.io..."
        flyctl logs
        ;;

    status)
        print_info "Checking app status..."
        flyctl status
        ;;

    ssh)
        print_info "Opening SSH connection to app..."
        flyctl ssh console
        ;;

    scale)
        INSTANCES=${2:-1}
        print_info "Scaling to $INSTANCES instance(s)..."
        flyctl scale count $INSTANCES
        ;;

    db-backup)
        print_info "Creating database backup..."
        BACKUP_FILE="tatteau-backup-$(date +%Y%m%d-%H%M%S).db"
        flyctl ssh console -C "cat /app/data/tatteau.db" > $BACKUP_FILE
        print_info "Backup saved to: $BACKUP_FILE"
        ;;

    db-restore)
        if [ -z "$2" ]; then
            print_error "Please provide backup file: ./deploy.sh db-restore <backup-file>"
            exit 1
        fi
        print_warn "This will overwrite the database. Are you sure? (yes/no)"
        read -r confirm
        if [ "$confirm" = "yes" ]; then
            print_info "Restoring database from $2..."
            cat $2 | flyctl ssh console -C "cat > /app/data/tatteau.db"
            print_info "Database restored! Restarting app..."
            flyctl apps restart
        else
            print_info "Restore cancelled"
        fi
        ;;

    destroy)
        print_warn "This will destroy the entire app. Are you sure? (yes/no)"
        read -r confirm
        if [ "$confirm" = "yes" ]; then
            flyctl apps destroy
        else
            print_info "Destroy cancelled"
        fi
        ;;

    *)
        echo "Tatteau Fly.io Deployment Tool"
        echo ""
        echo "Usage: $0 <command>"
        echo ""
        echo "Commands:"
        echo "  init        Initialize new Fly.io app"
        echo "  deploy      Deploy app to Fly.io"
        echo "  logs        Stream application logs"
        echo "  status      Check app status"
        echo "  ssh         Open SSH console to app"
        echo "  scale <n>   Scale to N instances"
        echo "  db-backup   Backup SQLite database"
        echo "  db-restore  Restore SQLite database"
        echo "  destroy     Destroy the Fly.io app"
        echo ""
        echo "Examples:"
        echo "  $0 init           # First time setup"
        echo "  $0 deploy         # Deploy changes"
        echo "  $0 scale 2        # Scale to 2 instances"
        echo "  $0 db-backup      # Backup database"
        ;;
esac
