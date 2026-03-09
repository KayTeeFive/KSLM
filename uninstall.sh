#!/usr/bin/env bash
set -e

# Stop and disable the systemd user service
if systemctl --user is-active --quiet kslm.service; then
    systemctl --user stop kslm.service
fi

if systemctl --user is-enabled --quiet kslm.service; then
    systemctl --user disable kslm.service
fi

# Remove the systemd unit file
UNIT_FILE="$HOME/.config/systemd/user/kslm.service"
if [ -f "$UNIT_FILE" ]; then
    rm -v "$UNIT_FILE"
    systemctl --user daemon-reload
fi

# Remove the script
SCRIPT_FILE="$HOME/.local/bin/kslm.py"
if [ -f "$SCRIPT_FILE" ]; then
    rm -v "$SCRIPT_FILE"
fi

echo "KSLM uninstalled successfully!"