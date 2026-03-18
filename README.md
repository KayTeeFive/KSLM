# KSLM – KDE Simple Layout Manager (SpareLayouts Hack)

**KSLM** is a small D-Bus daemon for KDE on Wayland/X11 that fixes issues with keyboard spare layouts.  
It provides a simple toggle method via D-Bus and integrates with KDE's keyboard layout system.  
⚠️ **Note:** This is a workaround ("hack") for KDE layout handling issues, not an official KDE feature.

---

## Features

- Toggle between user-defined keyboard layouts.
- Tracks KDE layout changes and updates its cache automatically.
- Exposes a D-Bus interface for programmatic layout switching.
- Logs activity to `systemd` journal.
- Works on KDE Wayland and X11 sessions.

---

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/KayTeeFive/KSLM.git
cd kslm
```

### 2. Install the script
```bash
./install.sh
```

This will:
- Copy `kslm.py` to `~/.local/bin/`.
- Copy the `kslm.service` systemd user unit to `~/.config/systemd/user/`.
- Enable and start the service using `systemctl --user`.

### 3. Uninstallation
```bash
./uninstall.sh
```

This will:
- Stop and disable the systemd user service.
- Remove the script from `~/.local/bin/`.
- Remove the systemd unit from `~/.config/systemd/user/`.

---

## Usage
Once installed, the daemon runs automatically in the background.

### Toggle layouts manually

You can toggle layouts via D-Bus:
```bash
qdbus org.kslm.LayoutDaemon /org/kslm/LayoutDaemon org.kslm.LayoutDaemon.toggle
```

Or from another script using Python:
```python
from pydbus import SessionBus

bus = SessionBus()
kslm = bus.get("org.kslm.LayoutDaemon", "/org/kslm/LayoutDaemon")
kslm.toggle()
```

### Configuration
The layouts to toggle are defined in:
```
~/.config/kslm.yml
```
Example:
```
layouts:
  - us
  - ua
```
The daemon will cycle through these layouts when toggled.

### Service control

If config changed, service restart required:
```bash
systemctl restart --user kslm.service
```

Service status:
```bash
systemctl status --user kslm.service
```

To start service:
```bash
systemctl start --user kslm.service
```

To stop service:
```bash
systemctl stop --user kslm.service
```
--- 

## Adding KSLM Toggle to KDE Keyboard Shortcuts

1. Open System Settings → Shortcuts → Custom Shortcuts.
2. Create a New → Global Shortcut → Command/URL.
3. Name it (e.g., Toggle Keyboard Layout).
4. Set a trigger key combination (e.g., Ctrl+Alt+Space).
5. Set the action to this command:
    ```
    qdbus org.kslm.LayoutDaemon /org/kslm/LayoutDaemon org.kslm.LayoutDaemon.toggle
    ```
6. Click Apply.

---

## Logging
All output is sent to systemd journal.

To follow logs in real time:
```bash
journalctl --user -u kslm.service -f
```

Or to view historical logs:
```bash
journalctl --user -u kslm.service
```

---
## Development

Requires Python 3, pydbus, PyYAML, and PyGObject.

Tested with KDE Plasma on Wayland and X11.

Modify kslm.py for custom behaviors (e.g., add more layouts, logging, or hotkeys).

---

## Disclaimer

This project is a **workaround** for KDE's layout management limitations.  

The reason this script exists is due to the **SpareLayout feature**, which works in X11 but does **not work in Wayland**.  
KDE plans to eventually drop X11 support, and there are no visible plans to fix or restore this feature in Wayland.  
- KWin Wayland transition requirements: [Requirements for dropping Xorg support](https://invent.kde.org/plasma/kwin/-/issues/202)  
- Stagnated bug/feature report: ["Spare Layouts" feature on Wayland](https://bugs.kde.org/show_bug.cgi?id=455431)
- Stagnated MR fix: [xkb: Enable Spare Layouts for Wayland (2)](https://invent.kde.org/plasma/kwin/-/merge_requests/5963)
- Known KDE Wayland issues: [Plasma/Wayland Known Significant Issues](https://community.kde.org/Plasma/Wayland_Known_Significant_Issues)

Use this script at your own risk. It does not officially patch KDE but provides a personal toggle solution until KDE implements a proper Wayland-compatible layout feature.

---

## License

GPL-3.0 license © 2026
