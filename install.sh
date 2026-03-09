#!/usr/bin/env bash
set -e

# Copying script
mkdir -p "$HOME/.local/bin"
cp bin/kslm.py "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/kslm.py"

# Copying systemd unit
mkdir -p "$HOME/.config/systemd/user"
cp systemd/kslm.service "$HOME/.config/systemd/user/"

# Reload daemon and restart service
systemctl --user daemon-reload
systemctl --user enable kslm.service
systemctl --user start kslm.service

echo "KSLM installed and running!"
