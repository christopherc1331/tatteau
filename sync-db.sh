#!/bin/bash
# Sync remote Fly.io database to local file for DataGrip

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Syncing database from Fly.io...${NC}"

# Download latest database
./deploy.sh db-backup > /dev/null 2>&1

# Find the latest backup
LATEST=$(ls -t tatteau-backup-*.db 2>/dev/null | head -1)

if [ -z "$LATEST" ]; then
    echo "Error: No backup file found"
    exit 1
fi

# Copy to consistent filename for DataGrip
cp "$LATEST" tatteau-live.db

echo -e "${GREEN}✓ Database synced!${NC}"
echo "  File: tatteau-live.db"
echo "  Size: $(du -h tatteau-live.db | cut -f1)"
echo ""
echo "Open 'tatteau-live.db' in DataGrip to query the latest production data"

# Optional: cleanup old backups (keep last 5)
ls -t tatteau-backup-*.db 2>/dev/null | tail -n +6 | xargs rm -f 2>/dev/null || true
