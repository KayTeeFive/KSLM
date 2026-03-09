#!/usr/bin/env bash
set -e

# Copying script
mkdir -p "$HOME/.local/bin"
cp target/release/kslm-rust "$HOME/.local/bin/"
chmod +x "$HOME/.local/bin/kslm-rust"

# Copying systemd unit
mkdir -p "$HOME/.config/systemd/user"
cp systemd/kslm-rust.service "$HOME/.config/systemd/user/"

# Reload daemon and restart service
systemctl --user daemon-reload
systemctl --user enable kslm-rust.service
systemctl --user start kslm-rust.service

echo "KSLM installed and running!"
