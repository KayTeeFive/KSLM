# KSLM – KDE Simple Layout Manager (Rust)

KDE Wayland does not support switching keyboard layouts via `xdotool` / `xkb-switch`.
KSLM is a minimal D-Bus daemon that subscribes to the KDE Keyboard API
and exposes a `toggle` method for cycling through layouts.

## Requirements

- Docker

## Project structure

```
.
├── src/
│   └── kslm-rust.rs        # Daemon source
├── systemd/
│   └── kslm-rust.service   # Systemd user service
├── Cargo.toml
├── Dockerfile              # Build environment (rust:1.89-slim-trixie)
├── build_binary.sh         # Compile binary inside Docker
├── build_env.sh            # Build Docker image
├── install.sh              # Install binary + systemd service
└── uninstall.sh            # Remove binary + systemd service
```

## Build

**Step 1.** Build the Docker image (only once):

```bash
./build_env.sh
```

This creates the `kslm-builder:260309` image based on `rust:1.89-slim-trixie`
with `libdbus-1-dev` and `libglib2.0-dev`.

**Step 2.** Compile the binary:

```bash
./build_binary.sh
```

Binary will be at `target/release/kslm-rust`.

## Install

```bash
./install.sh
```

This will:
- Copy `target/release/kslm-rust` → `~/.local/bin/kslm-rust`
- Copy `systemd/kslm-rust.service` → `~/.config/systemd/user/`
- Enable and start the service via `systemctl --user`

## Uninstall

```bash
./uninstall.sh
```

This will stop and disable the service, then remove the binary and unit file.

---

## Configuration

`~/.config/kslm.yml` is created automatically on first run:

```yaml
layouts:
  - us
  - ua
```

List layouts in the desired cycling order.
Layout codes must match those in KDE Settings → Input Devices → Keyboard → Layouts.

## Switching layouts

Call `toggle` via D-Bus:

```bash
dbus-send --session --type=method_call \
  --dest=org.kslm.LayoutDaemon \
  /org/kslm/LayoutDaemon \
  org.kslm.LayoutDaemon.toggle
```
Or
```bash
qdbus org.kslm.LayoutDaemon /org/kslm/LayoutDaemon toggle
```

Bind this command to a shortcut in KDE Settings → Shortcuts → Custom Shortcuts.

---

## Adding KSLM Toggle to KDE Keyboard Shortcuts

1. Open System Settings → Shortcuts → Custom Shortcuts.
2. Create a New → Global Shortcut → Command/URL.
3. Name it (e.g., Toggle Keyboard Layout).
4. Set a trigger key combination (e.g., Ctrl+Alt+Space).
5. Set the action to this command:
    ```
    qdbus org.kslm.LayoutDaemon /org/kslm/LayoutDaemon toggle
    ```
6. Click Apply.

---

## Log level

Configured in `systemd/kslm-rust.service`:

```ini
Environment=RUST_LOG=kslm=info    # default
Environment=RUST_LOG=kslm=debug   # verbose (signals, layout changes)
```

After changing, reinstall the service:

```bash
./install.sh
```

Or to test without reinstalling:

```bash
RUST_LOG=kslm=debug ~/.local/bin/kslm-rust
```