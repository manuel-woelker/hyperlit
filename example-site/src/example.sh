#!/bin/bash

# 📖 Why this script uses a log file instead of stdout
# This script runs as a cron job and output needs to be preserved for debugging.
# Writing to a log file ensures we can check historical runs even if the cron
# daemon's email is not configured. The log rotation prevents disk space issues.

LOG_FILE="/var/log/backup.log"
SOURCE_DIR="/home/user/documents"
BACKUP_DIR="/backup"

# Function to log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

log "Starting backup process"

# Create timestamped backup
timestamp=$(date '+%Y%m%d_%H%M%S')
backup_file="$BACKUP_DIR/backup_$timestamp.tar.gz"

tar -czf "$backup_file" "$SOURCE_DIR" 2>> "$LOG_FILE"

if [ $? -eq 0 ]; then
    log "Backup successful: $backup_file"
    
    # Clean up old backups (keep last 7 days)
    find "$BACKUP_DIR" -name "backup_*.tar.gz" -mtime +7 -delete
    log "Cleaned up old backups"
else
    log "Backup failed - check log for details"
    exit 1
fi

log "Backup process completed"