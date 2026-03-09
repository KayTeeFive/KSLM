#!/usr/bin/env bash
set -e

# Stop and disable the systemd user service
if systemctl --user is-active --quiet kslm-rust.service; then
    systemctl --user stop kslm-rust.service
fi

if systemctl --user is-enabled --quiet kslm-rust.service; then
    systemctl --user disable kslm-rust.service
fi

# Remove the systemd unit file
UNIT_FILE="$HOME/.config/systemd/user/kslm-rust.service"
if [ -f "$UNIT_FILE" ]; then
    rm -v "$UNIT_FILE"
    systemctl --user daemon-reload
fi

# Remove the script
SCRIPT_FILE="$HOME/.local/bin/kslm-rust"
if [ -f "$SCRIPT_FILE" ]; then
    rm -v "$SCRIPT_FILE"
fi

echo "KSLM uninstalled successfully!"
