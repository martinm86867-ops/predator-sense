#!/usr/bin/env python3
"""
Acer Predator Keyboard Backlight & RGB Hardware Controller
Supports static 4-zone illumination, dynamic hardware effects, and boot-persistence
interfacing with /dev/acer-gkbbl-0 and /dev/acer-gkbbl-static-0.
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path

PAYLOAD_SIZE = 16
CHARACTER_DEVICE = "/dev/acer-gkbbl-0"

PAYLOAD_SIZE_STATIC_MODE = 4
CHARACTER_DEVICE_STATIC = "/dev/acer-gkbbl-static-0"

DEFAULT_CONFIG_DIR = Path.home() / ".config" / "predator-sense"
DEFAULT_CONFIG_PATH = DEFAULT_CONFIG_DIR / "config.json"

MODE_MAP = {
    "static": 0,
    "breath": 1,
    "neon": 2,
    "wave": 3,
    "shifting": 4,
    "zoom": 5,
    "meteor": 6,
    "twinkling": 7,
}

REV_MODE_MAP = {v: k.capitalize() for k, v in MODE_MAP.items()}

def parse_mode_int(val):
    if isinstance(val, int):
        return val
    return MODE_MAP.get(str(val).lower(), 3)

def load_app_config(custom_home=None):
    if custom_home:
        cfg_path = Path(custom_home) / ".config" / "predator-sense" / "config.json"
    else:
        cfg_path = DEFAULT_CONFIG_PATH
    if cfg_path.exists():
        try:
            with open(cfg_path, "r", encoding="utf-8") as f:
                return json.load(f), cfg_path
        except Exception as e:
            print(f"Warning: Could not read {cfg_path}: {e}", file=sys.stderr)
    return {}, cfg_path

def save_app_config(cfg, cfg_path):
    try:
        cfg_path.parent.mkdir(parents=True, exist_ok=True)
        with open(cfg_path, "w", encoding="utf-8") as f:
            json.dump(cfg, f, indent=2)
    except Exception as e:
        print(f"Warning: Could not save {cfg_path}: {e}", file=sys.stderr)

def apply_static(zones_colors, brightness=100):
    if not os.path.exists(CHARACTER_DEVICE_STATIC) or not os.path.exists(CHARACTER_DEVICE):
        return False
    # zones_colors is a dict or list: {1: (r,g,b), 2: ..., 3: ..., 4: ...}
    for z, (r, g, b) in zones_colors.items():
        payload = [0] * PAYLOAD_SIZE_STATIC_MODE
        payload[0] = 1 << (z - 1)
        payload[1] = max(0, min(255, r))
        payload[2] = max(0, min(255, g))
        payload[3] = max(0, min(255, b))
        with open(CHARACTER_DEVICE_STATIC, "wb") as cd:
            cd.write(bytes(payload))
    
    # Commit brightness via WMI dynamic packet
    payload = [0] * PAYLOAD_SIZE
    payload[2] = max(0, min(100, brightness))
    payload[8] = 1 if brightness > 0 else 0  # KLES enable flag in ACPI WMBH Method 0x14
    payload[9] = 1
    with open(CHARACTER_DEVICE, "wb") as cd:
        cd.write(bytes(payload))
    return True

def apply_dynamic(mode, speed=4, brightness=100, direction=1, r=0, g=255, b=255):
    if not os.path.exists(CHARACTER_DEVICE):
        return False
    mode_int = parse_mode_int(mode)
    payload = [0] * PAYLOAD_SIZE
    payload[0] = mode_int
    payload[1] = max(0, min(9, speed))
    payload[2] = max(0, min(100, brightness))
    payload[3] = 8 if mode_int == 3 else 0
    payload[4] = direction
    payload[5] = max(0, min(255, r))
    payload[6] = max(0, min(255, g))
    payload[7] = max(0, min(255, b))
    payload[8] = 1 if brightness > 0 else 0  # KLES enable flag in ACPI WMBH Method 0x14
    payload[9] = 1

    with open(CHARACTER_DEVICE, "wb") as cd:
        cd.write(bytes(payload))
    return True

def restore_from_config(custom_home=None):
    cfg, cfg_path = load_app_config(custom_home)
    if not cfg:
        # Fallback to Wave effect at 100% brightness
        apply_dynamic(3, speed=4, brightness=100)
        print("Restored default dynamic Wave effect (no config found).")
        return

    brightness = cfg.get("rgb_brightness", 100)
    is_static = cfg.get("rgb_is_static", False)

    if is_static:
        static_zones = cfg.get("rgb_static_zones")
        zones_colors = {}
        if static_zones and isinstance(static_zones, list):
            for item in static_zones:
                z = item.get("zone", 1)
                zones_colors[z] = (item.get("red", 255), item.get("green", 255), item.get("blue", 255))
        if len(zones_colors) < 4:
            # Fill remaining with dynamic_last color or white
            dyn = cfg.get("rgb_dynamic_last") or {}
            def_r = dyn.get("red", 255)
            def_g = dyn.get("green", 255)
            def_b = dyn.get("blue", 255)
            for z in range(1, 5):
                if z not in zones_colors:
                    zones_colors[z] = (def_r, def_g, def_b)
        apply_static(zones_colors, brightness=brightness)
        print(f"Boot-restored static RGB profile across 4 zones at {brightness}% brightness.")
    else:
        dyn = cfg.get("rgb_dynamic_last") or {}
        mode = parse_mode_int(dyn.get("mode", 3))
        speed = dyn.get("speed", 4)
        direction = 1 if dyn.get("direction", "RightToLeft") == "RightToLeft" else 2
        r = dyn.get("red", 0)
        g = dyn.get("green", 255)
        b = dyn.get("blue", 255)
        apply_dynamic(mode, speed=speed, brightness=brightness, direction=direction, r=r, g=g, b=b)
        print(f"Boot-restored dynamic effect '{REV_MODE_MAP.get(mode, mode)}' at {brightness}% brightness.")

def main():
    parser = argparse.ArgumentParser(description="Acer Predator Keyboard Backlight Controller & Boot Restore")
    parser.add_argument("--boot-apply", nargs="?", const="", help="Reapply persisted RGB profile from user home directory at boot")
    parser.add_argument("-m", "--mode", type=str, default="3", help="0: Static, 1: Breath, 2: Neon, 3: Wave, 4: Shifting, 5: Zoom (or mode name)")
    parser.add_argument("-z", "--zone", type=int, default=0, help="Zone ID (1-4, or 0 for all zones)")
    parser.add_argument("-s", "--speed", type=int, default=4, help="Speed (0-9)")
    parser.add_argument("-b", "--brightness", type=int, default=-1, help="Brightness (0-100)")
    parser.add_argument("-d", "--direction", type=int, default=1, help="1: Right to Left, 2: Left to Right")
    parser.add_argument("-cR", "--red", type=int, default=255, help="Red (0-255)")
    parser.add_argument("-cG", "--green", type=int, default=255, help="Green (0-255)")
    parser.add_argument("-cB", "--blue", type=int, default=255, help="Blue (0-255)")
    parser.add_argument("cmd", nargs="?", help="Subcommand: on, off, brightness, color, mode, restore")
    parser.add_argument("cmd_args", nargs="*", help="Arguments for subcommand")

    args = parser.parse_args()

    if args.boot_apply is not None:
        restore_from_config(args.boot_apply if args.boot_apply else None)
        sys.exit(0)

    # Subcommand compatibility
    if args.cmd:
        sub = args.cmd.lower()
        if sub == "off":
            apply_dynamic(0, brightness=0, r=0, g=0, b=0)
            cfg, path = load_app_config()
            cfg["rgb_brightness"] = 0
            save_app_config(cfg, path)
            print("Backlight turned off.")
            sys.exit(0)
        elif sub == "on":
            br = int(args.cmd_args[0]) if args.cmd_args else 100
            restore_from_config()
            cfg, path = load_app_config()
            cfg["rgb_brightness"] = br
            save_app_config(cfg, path)
            print(f"Backlight turned on at {br}%.")
            sys.exit(0)
        elif sub == "brightness":
            if not args.cmd_args:
                print("Usage: kb-backlight brightness <0-100>")
                sys.exit(1)
            br = int(args.cmd_args[0])
            cfg, path = load_app_config()
            cfg["rgb_brightness"] = br
            save_app_config(cfg, path)
            restore_from_config()
            print(f"Backlight brightness set to {br}% and persisted.")
            sys.exit(0)
        elif sub == "color":
            if len(args.cmd_args) < 3:
                print("Usage: kb-backlight color <R> <G> <B> [brightness]")
                sys.exit(1)
            r, g, b = int(args.cmd_args[0]), int(args.cmd_args[1]), int(args.cmd_args[2])
            br = int(args.cmd_args[3]) if len(args.cmd_args) > 3 else 100
            zones = {z: (r, g, b) for z in range(1, 5)}
            apply_static(zones, brightness=br)
            cfg, path = load_app_config()
            cfg["rgb_is_static"] = True
            cfg["rgb_brightness"] = br
            cfg["rgb_static_zones"] = [{"zone": z, "red": r, "green": g, "blue": b} for z in range(1, 5)]
            save_app_config(cfg, path)
            print(f"Static color RGB({r},{g},{b}) applied to all zones and persisted.")
            sys.exit(0)
        elif sub == "restore":
            restore_from_config()
            sys.exit(0)

    # Legacy flag compatibility
    try:
        mode_val = int(args.mode)
    except ValueError:
        mode_val = parse_mode_int(args.mode)

    br = args.brightness if args.brightness >= 0 else 100

    if mode_val == 0:
        target_zones = [1, 2, 3, 4] if args.zone == 0 else [args.zone]
        zones_colors = {z: (args.red, args.green, args.blue) for z in target_zones}
        apply_static(zones_colors, brightness=br)
        cfg, path = load_app_config()
        cfg["rgb_is_static"] = True
        cfg["rgb_brightness"] = br
        if "rgb_static_zones" not in cfg or not isinstance(cfg["rgb_static_zones"], list):
            cfg["rgb_static_zones"] = [{"zone": z, "red": args.red, "green": args.green, "blue": args.blue} for z in range(1, 5)]
        else:
            for item in cfg["rgb_static_zones"]:
                if item.get("zone") in target_zones:
                    item["red"] = args.red
                    item["green"] = args.green
                    item["blue"] = args.blue
        save_app_config(cfg, path)
        print(f"Static color RGB({args.red},{args.green},{args.blue}) applied to zone(s) {target_zones} at brightness {br}% and persisted.")
    else:
        apply_dynamic(mode_val, speed=args.speed, brightness=br, direction=args.direction, r=args.red, g=args.green, b=args.blue)
        cfg, path = load_app_config()
        cfg["rgb_is_static"] = False
        cfg["rgb_brightness"] = br
        cfg["rgb_dynamic_last"] = {
            "mode": REV_MODE_MAP.get(mode_val, "Wave"),
            "speed": args.speed,
            "brightness": br,
            "direction": "RightToLeft" if args.direction == 1 else "LeftToRight",
            "red": args.red,
            "green": args.green,
            "blue": args.blue,
        }
        save_app_config(cfg, path)
        print(f"Dynamic effect '{REV_MODE_MAP.get(mode_val, mode_val)}' applied at brightness {br}% and persisted.")

if __name__ == "__main__":
    main()
