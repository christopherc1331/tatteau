#!/bin/bash
# Copy database from FileBrowser volume to tatteau-app volume via SSH

set -e

echo "Step 1: Downloading database from FileBrowser service..."
mkdir -p /tmp/railway-transfer
railway ssh -s Filebrowser "cat /data/tatteau.db" > /tmp/railway-transfer/tatteau.db

if [ ! -s /tmp/railway-transfer/tatteau.db ]; then
    echo "Error: Downloaded file is empty"
    exit 1
fi

SIZE=$(du -h /tmp/railway-transfer/tatteau.db | cut -f1)
echo "Downloaded: $SIZE"

echo "Step 2: Uploading database to tatteau-app service..."
railway ssh "cat > /app/data/tatteau.db && chmod 644 /app/data/tatteau.db && ls -lh /app/data/tatteau.db" < /tmp/railway-transfer/tatteau.db

echo "Step 3: Verifying upload..."
railway ssh "sqlite3 /app/data/tatteau.db 'SELECT COUNT(*) FROM artists;'"

echo "Success! Database copied between volumes."
echo "You can now delete the FileBrowser service."

# Cleanup
rm -f /tmp/railway-transfer/tatteau.db
