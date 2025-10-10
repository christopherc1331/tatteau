#!/bin/bash
# Sync remote Railway database to local file for DataGrip

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Syncing database from Railway...${NC}"

# Check if railway CLI is available
if ! command -v railway &> /dev/null; then
    echo -e "${RED}Error: Railway CLI not installed${NC}"
    echo "Install with: npm i -g @railway/cli"
    exit 1
fi

# Check if logged in
if ! railway whoami &> /dev/null; then
    echo -e "${RED}Error: Not logged into Railway${NC}"
    echo "Run: railway login"
    exit 1
fi

# Download latest database using deploy.sh
./deploy.sh db-backup > /dev/null 2>&1

# Find the latest backup
LATEST=$(ls -t tatteau-backup-*.db 2>/dev/null | head -1)

if [ -z "$LATEST" ]; then
    echo -e "${RED}Error: No backup file found${NC}"
    exit 1
fi

# Verify backup is not empty
if [ ! -s "$LATEST" ]; then
    echo -e "${RED}Error: Backup file is empty${NC}"
    rm -f "$LATEST"
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
