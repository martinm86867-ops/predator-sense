#!/usr/bin/env python3
import argparse
import json
import os
import sys
from pathlib import Path

PAYLOAD_SIZE = 16
CHARACTER_DEVICE = "/dev/acer-gkbbl-0"

PAYLOAD_SIZE_STATIC_MODE = 4
CHARACTER_DEVICE_STATIC = "/dev/acer-gkbbl-static-0"

CONFIG_DIRECTORY = str(Path.home()) + "/.config/predator/saved profiles"
path = Path(CONFIG_DIRECTORY)
path.mkdir(parents=True, exist_ok=True)

parser = argparse.ArgumentParser(description="Acer Predator Keyboard Backlight Controller", formatter_class=argparse.RawTextHelpFormatter)
parser.add_argument('-m', type=int, dest='mode', default=3, help='0: Static, 1: Breath, 2: Neon, 3: Wave, 4: Shifting, 5: Zoom')
parser.add_argument('-z', type=int, dest='zone', default=1, help='Zone ID (1-4, or 0 for all zones in static mode)')
parser.add_argument('-s', type=int, dest='speed', default=4, help='Speed (0-9)')
parser.add_argument('-b', type=int, dest='brightness', default=100, help='Brightness (0-100)')
parser.add_argument('-d', type=int, dest='direction', default=1, help='1: Right to Left, 2: Left to Right')
parser.add_argument('-cR', type=int, dest='red', default=255, help='Red (0-255)')
parser.add_argument('-cG', type=int, dest='green', default=255, help='Green (0-255)')
parser.add_argument('-cB', type=int, dest='blue', default=255, help='Blue (0-255)')
parser.add_argument('-save', help='Save profile name')
parser.add_argument('-load', help='Load profile name')
parser.add_argument('-list', action='store_true', help='List saved profiles')

args = parser.parse_args()

if args.list:
    print("Saved profiles:")
    for filepath in list(path.glob('*.json')):
        print(f"\t{filepath.stem}")
    sys.exit(0)

if args.load:
    profile_file = Path(f"{CONFIG_DIRECTORY}/{args.load}.json")
    if profile_file.exists():
        with open(profile_file, 'rt') as f:
            t_args = argparse.Namespace()
            t_args.__dict__.update(json.load(f))
            args = parser.parse_args(namespace=t_args)

if args.save:
    with open(f"{CONFIG_DIRECTORY}/{args.save}.json", 'wt') as f:
        d = vars(args).copy()
        d.pop('save', None)
        d.pop('load', None)
        d.pop('list', None)
        json.dump(d, f, indent=4)

if not os.path.exists(CHARACTER_DEVICE):
    print(f"Error: {CHARACTER_DEVICE} not found. Is 'facer' kernel module loaded?")
    sys.exit(1)

if args.mode == 0:
    # Static mode: apply to specified zone or all 4 zones
    zones_to_apply = [1, 2, 3, 4] if args.zone == 0 else [args.zone]
    for z in zones_to_apply:
        payload = [0] * PAYLOAD_SIZE_STATIC_MODE
        payload[0] = 1 << (z - 1)
        payload[1] = max(0, min(255, args.red))
        payload[2] = max(0, min(255, args.green))
        payload[3] = max(0, min(255, args.blue))
        with open(CHARACTER_DEVICE_STATIC, 'wb') as cd:
            cd.write(bytes(payload))

    # Commit static brightness to WMI
    payload = [0] * PAYLOAD_SIZE
    payload[2] = max(0, min(100, args.brightness))
    payload[9] = 1
    with open(CHARACTER_DEVICE, 'wb') as cd:
        cd.write(bytes(payload))
    print(f"Static color RGB({args.red},{args.green},{args.blue}) applied to zone(s) {zones_to_apply} at brightness {args.brightness}%.")

else:
    # Dynamic mode
    payload = [0] * PAYLOAD_SIZE
    payload[0] = args.mode
    payload[1] = max(0, min(9, args.speed))
    payload[2] = max(0, min(100, args.brightness))
    payload[3] = 8 if args.mode == 3 else 0
    payload[4] = args.direction
    payload[5] = max(0, min(255, args.red))
    payload[6] = max(0, min(255, args.green))
    payload[7] = max(0, min(255, args.blue))
    payload[9] = 1

    with open(CHARACTER_DEVICE, 'wb') as cd:
        cd.write(bytes(payload))
    mode_names = {1: "Breath", 2: "Neon", 3: "Wave", 4: "Shifting", 5: "Zoom"}
    print(f"Dynamic effect '{mode_names.get(args.mode, args.mode)}' applied: speed={args.speed}, brightness={args.brightness}%.")
