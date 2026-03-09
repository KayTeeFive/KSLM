#!/usr/bin/env python3

import os
import yaml
from pydbus import SessionBus
from gi.repository import GLib, Gio
import logging

# ---------------------------
# Logging setup
# ---------------------------
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s"
)
log = logging.getLogger("KSLM")

CONFIG_PATH = os.path.expanduser("~/.config/kslm.yml")
DEFAULT_LAYOUTS = ["us", "ua"]

LAYOUTS = []
LAYOUT_MAP = {}
CFG_LAYOUTS = []

# ---------------------------
# Load configuration from YAML
# ---------------------------
def load_config():
    global CFG_LAYOUTS

    # Create config file with defaults if missing
    if not os.path.exists(CONFIG_PATH):
        os.makedirs(os.path.dirname(CONFIG_PATH), exist_ok=True)
        with open(CONFIG_PATH, "w") as f:
            yaml.dump({"layouts": DEFAULT_LAYOUTS}, f)

    try:
        with open(CONFIG_PATH) as f:
            cfg = yaml.safe_load(f)
            CFG_LAYOUTS = cfg.get("layouts", DEFAULT_LAYOUTS)
    except Exception as e:
        log.error(f"Config error: {e}")
        CFG_LAYOUTS = DEFAULT_LAYOUTS

# ---------------------------
# Connect to KDE keyboard D-Bus API
# ---------------------------
bus = SessionBus()
keyboard = bus.get("org.kde.keyboard", "/Layouts")

# ---------------------------
# Refresh layout cache
# ---------------------------
def refresh_layouts():
    global LAYOUTS, LAYOUT_MAP
    try:
        new_layouts = []
        for i, (layout, variant, name) in enumerate(keyboard.getLayoutsList()):
            new_layouts.append({
                "index": i,
                "layout": layout,
                "variant": variant,
                "name": name
            })

        # Update global cache if layouts changed
        if new_layouts != LAYOUTS:
            LAYOUTS = new_layouts
            LAYOUT_MAP = {l["layout"]: l["index"] for l in LAYOUTS}
            log.info("Layout map updated:")
            for l in LAYOUTS:
                log.info(f'{l["index"]:2}  {l["layout"]:3}  {l["name"]}')

    except Exception as e:
        log.error(f"Failed to refresh layouts: {e}")

# ---------------------------
# Layout helpers
# ---------------------------
def get_current_layout():
    # Get current layout index from KDE and map to layout code
    index = keyboard.getLayout()
    if 0 <= index < len(LAYOUTS):
        return LAYOUTS[index]["layout"]
    return LAYOUTS[0]["layout"]

def set_layout(index):
    # Set layout by index
    keyboard.setLayout(index)

# ---------------------------
# Toggle logic
# ---------------------------
def next_layout():
    current = get_current_layout()
    if current not in CFG_LAYOUTS:
        target = CFG_LAYOUTS[0]
    else:
        i = CFG_LAYOUTS.index(current)
        target = CFG_LAYOUTS[(i + 1) % len(CFG_LAYOUTS)]

    target_index = LAYOUT_MAP.get(target)
    if target_index is not None:
        set_layout(target_index)
        log.info(f"Switched layout to: {target}")

# ---------------------------
# D-Bus service
# ---------------------------
class KSLM:
    # Method exposed via D-Bus
    def toggle(self):
        next_layout()

interface_xml = """<node>
  <interface name="org.kslm.LayoutDaemon">
    <method name="toggle"/>
  </interface>
</node>"""

bus.request_name("org.kslm.LayoutDaemon")
bus.register_object("/org/kslm/LayoutDaemon", KSLM(), [interface_xml])

# ---------------------------
# Subscribe to KDE layout signals using Gio
# ---------------------------
gconnection = Gio.bus_get_sync(Gio.BusType.SESSION, None)

def on_layout_list_changed(connection, sender_name, object_path, interface_name, signal_name, parameters):
    # Called when the list of layouts changes
    log.debug("Layout list changed signal received!")
    refresh_layouts()

def on_layout_changed(connection, sender_name, object_path, interface_name, signal_name, parameters):
    # Called when the current layout changes
    index = parameters.unpack()[0]
    if 0 <= index < len(LAYOUTS):
        log.debug(f"Layout changed signal: {LAYOUTS[index]["layout"]}")
    else:
        log.debug(f"Layout changed signal: {index}")

# Subscribe to layout list change signal
gconnection.signal_subscribe(
    sender="org.kde.keyboard",
    interface_name="org.kde.KeyboardLayouts",
    member="layoutListChanged",
    object_path="/Layouts",
    arg0=None,
    flags=Gio.DBusSignalFlags.NONE,
    callback=on_layout_list_changed
)

# Subscribe to current layout change signal
gconnection.signal_subscribe(
    sender="org.kde.keyboard",
    interface_name="org.kde.KeyboardLayouts",
    member="layoutChanged",
    object_path="/Layouts",
    arg0=None,
    flags=Gio.DBusSignalFlags.NONE,
    callback=on_layout_changed
)

# ---------------------------
# Startup
# ---------------------------
load_config()
refresh_layouts()

log.info("Layout daemon running...")
GLib.MainLoop().run()