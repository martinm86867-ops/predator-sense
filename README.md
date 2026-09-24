# Predator Sense for Linux

<p align="center">
  <img src="predator-sense-gui/resources/logo-256.png" width="140" alt="Predator Sense logo">
</p>

<p align="center">
  <b>Hardware control for Acer gaming laptops — kept alive past official support.</b><br>
  <i>Turbo &bull; Thermal management &bull; RGB &bull; Fan control &bull; Performance profiles</i>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/GTK-4-blue?logo=gtk" alt="GTK4">
  <img src="https://img.shields.io/badge/License-GPL--3.0-green" alt="GPL-3.0">
  <img src="https://img.shields.io/badge/Platform-Linux-yellow?logo=linux" alt="Linux">
</p>

---

## What this is

An unofficial, from-scratch reverse-engineering effort to keep Acer Predator/Helios/Nitro laptops fully usable on Linux. It ships a Linux kernel module (facer) plus a Rust + GTK4 desktop application for RGB backlighting, turbo mode, thermal/fan management, performance profiles, and hardware monitoring.

The project began as a fork of [acer-predator-turbo-and-rgb-keyboard-linux-module](https://github.com/JafarAkhondali/acer-predator-turbo-and-rgb-keyboard-linux-module) by [JafarAkhondali](https://github.com/JafarAkhondali), and grew into its own thing: the focus is now on **models the firmware never officially supported on Linux**, with honest reporting of what is verified to work versus what is still best-effort.

## Highlights

- **Turbo and thermal control, reverse-engineered** — the gaming WMI interface (`GUID 7A4DDFE7-…`) was decoded far enough to drive the physical Predator/Turbo key, OC mode, and fan Auto/Max/custom-% on hardware upstream never enabled (e.g. the **PH317-55**).
- **Fan state that doesn't fight itself** — manual modes, the auto fan curve, the power-source policy, and the physical key are mutually exclusive and persist correctly, instead of silently overwriting each other.
- **Honest capability detection** — every feature is detected at runtime and labeled *verified / unverified / incompatible* per model. Unsupported things say so, rather than pretending to work.
- **Per-profile dynamic theming** — the entire UI (CSS, hand-drawn chrome, icons) recolors to the active mode: Quiet teal, Balanced cyan, Performance orange, Turbo red, Eco green.
- **Built-in hardware report** — a page that documents what was detected and how confident the tool is in each control, for anyone keeping an unsupported machine running.

## Features

| Feature | Description |
|---------|-------------|
| **Cooling hub** | Performance modes, fan control, and the GPU dashboard in one tabbed page |
| **Performance profiles** | Quiet / Balanced / Performance / Turbo, plus a battery-only Eco tier |
| **Fan control** | Live RPM with animated fans, CoolBoost, Auto/Max, and per-fan custom % |
| **Software fan curve** | Temperature-driven PWM curve with user-editable breakpoints |
| **Dashboard** | Live instrument cluster + system specs (CPU, GPU, RAM, storage, network, OS) |
| **Temperatures** | Gauges for CPU, GPU, system, NVMe, Wi-Fi and RAM |
| **Usage** | CPU / GPU / memory / storage with top processes |
| **Network** | Real-time download/upload graphs with peak tracking and auto interface detection |
| **Graphs** | CPU/GPU history charts with min/max tracking |
| **RGB keyboard** | Static per-zone (4 zones) and dynamic effects over WMI, or natively over I2C/USB-HID |
| **RGB cover logo** | Power, color, brightness, and effects for the display-lid emblem |
| **GPU dashboard** | NVIDIA metrics and a power-limit (TGP) slider |
| **Battery** | Charge stats, cycles, health, and charge-limit modes |
| **AI assistant** 🧪 | Opt-in, local (Ollama) assistant with a fixed, validated action set |
| **Hardware report** | Detected capabilities, per-control confidence, and reverse-engineering notes |
| **System tray + hotkey** | Minimize to tray; the PredatorSense key opens the app |
| **Auto capability detection** | Unsupported features are reported instead of erroring |
| **DKMS** | Kernel modules rebuild automatically across kernel upgrades |
| **Internationalization** | English, Portuguese, Spanish, Chinese, Japanese, Russian, German, Italian, Turkish |

---

## Disclaimer

> **Use at your own risk.** This is an **unofficial** project; Acer was not involved. The kernel module drives low-level WMI/ACPI interfaces that have not been tested on every laptop series. The authors are not responsible for any damage to your hardware.
>
> All trademarks, product names, and logos belong to their respective owners. This project is not affiliated with, endorsed by, or sponsored by Acer Inc.

---

## Installation

### Prebuilt installer

```console
curl --fail --location https://github.com/cleyton1986/predator-sense/releases/latest/download/predator-sense-installer --output predator-sense-installer
chmod +x predator-sense-installer
sudo ./predator-sense-installer --install
```

The installer, privileged helper, hotkey listener, and tray service are all one Rust multicall binary.

### Build from source

**Prerequisites** (Debian/Ubuntu shown; adjust for other distros):

```console
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config build-essential \
    gcc make dkms curl tar linux-headers-$(uname -r)
```

Install Rust, then build and install:

```console
git clone https://github.com/cleyton1986/predator-sense.git
cd predator-sense/predator-sense-gui

cargo build --release
cargo build --release --manifest-path installer/Cargo.toml

sudo installer/target/release/predator-sense-installer --install
/opt/predator-sense/predator-sense
```

---

## Usage

### Performance profiles

| Profile | Intel EPP | Min. performance | GPU Power | Fan | Use case |
|---------|-----------|------------------|-----------|-----|----------|
| **Eco** | power | 5% | 25W | Auto | Maximum battery life (battery-only) |
| **Quiet** | power | 10% | 40W | Auto | Silent work |
| **Balanced** | balance_performance | 17% | 80W | Auto | General use |
| **Performance** | performance | 50% | 100W | Max | Gaming |
| **Turbo** | 0 (kernel-forced) | 100% | 110W | Max | Maximum performance |

Selecting a profile also applies its fan mode. GPU power limits are applied best-effort via `nvidia-smi -pl`.

### Keyboard RGB

1. Open **Lighting** in the sidebar.
2. Choose **Static** (per-zone colors) or **Dynamic** (effects).
3. Adjust colors/effects and speed, then **Apply**.

### GPU Dashboard

Real-time NVIDIA monitoring: temperature, utilization, VRAM, power draw, clocks, P-State, PCIe link info, and VBIOS version.

### AI Assistant (beta)

Opt-in, local, powered by [Ollama](https://ollama.com):

1. Install Ollama separately ([instructions](https://ollama.com/download/linux)).
2. Download a model from the built-in manager (`smollm2:1.7b` or larger).
3. Enable the assistant in Settings and choose **Auto-apply** or **Always confirm**.

---

## Compatibility

Legend: ✅ tested & working · 🟡 implemented, not tested · 🧪 experimental · ❌ not working · `-` not applicable

| Product Name | Turbo (Impl.) | Turbo (Tested) | RGB (Impl.) | RGB (Tested) | Fan RPM read | Fan profiles | Fan PWM % |
|--------------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| AN16S-61 | - | - | ✅ | ✅ | ❌ | - | ❌ |
| AN515-45 | - | - | ✅ | ✅ | ❌ | - | ❌ |
| AN515-55 | - | - | ✅ | ✅ | ❌ | - | ❌ |
| AN515-56 | - | - | ✅ | ✅ | ❌ | - | ❌ |
| AN515-57 | - | - | ✅ | ✅ | ❌ | - | ❌ |
| AN515-58 | ✅ | 🟡 | ✅ | ✅ | 🟡 | 🟡 | 🧪 |
| AN517-41 | - | - | ✅ | ✅ | ❌ | - | ❌ |
| PH16-71 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | - | ❌ |
| PH16-72 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | 🟡 | 🧪 |
| PH315-52 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| PH315-53 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| **PH315-54** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| PH315-55 | ✅ | 🟡 | ✅ | ❌ | 🟡 | - | ❌ |
| PH317-53 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| PH317-54 | ✅ | ✅ | ✅ | 🟡 | ✅ | - | 🧪 |
| **PH317-55** | ✅ | ✅ | ✅ | ✅ | ✅ | - | ✅ |
| PH317-56 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | - | ❌ |
| PH517-51 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | - | ❌ |
| PH517-52 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | - | ❌ |
| PH517-61 | ✅ | 🟡 | ✅ | ✅ | 🟡 | - | ❌ |
| PHN16-71 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | - | ❌ |
| PHN16S-71 | ✅ | ✅ | ✅ | ✅ | ✅ | - | ❌ |
| PHN16-72 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🧪 |
| **PHN16-73** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| PHN18-71 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| PT314-51 | ❌ | ❌ | ✅ | ✅ | 🟡 | - | ❌ |
| PT314-52s | ✅ | ✅ | ✅ | 🟡 | 🟡 | - | ❌ |
| PT315-51 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| PT316-51 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| PT515-51 | ✅ | ✅ | ✅ | ✅ | 🟡 | - | ❌ |
| PT516-52s | ✅ | 🟡 | ✅ | ✅ | 🟡 | - | ❌ |
| PT917-71 | ✅ | 🟡 | ✅ | 🟡 | 🟡 | - | ❌ |

> If your model is not listed, it may still work — the kernel module detects compatible WMI interfaces automatically. Open an issue mentioning your model so the table can be updated.

---

## Reverse engineering notes

### PH317-55 (Predator Helios 300 2021)

This is the fork's primary target. Its firmware exposes the full gaming WMI interface (`GUID 7A4DDFE7-5B5D-40B4-8595-4408E0CC7F56`, object `"BH"` / `"BG"` → `WMBH` → `WSMI` → EC port `0xD0`), but upstream `facer.c` had no DMI quirk for it, so turbo, fan control, and reliable RGB were never enabled.

- **Turbo button** — added `quirk_acer_predator_ph317_55` with `turbo = 1`. The physical key toggles OC mode (`0x205`/`0x207`), turbo fan (method 14) and the turbo LED over the gaming WMI interface. Verified: fans ramp from ~3.5k to ~8k RPM.
- **Fan control** — added `pwm = 1` so Auto/Max and per-fan manual % route through the WMI-backed hwmon PWM path (methods 14–17) instead of the raw EC `0x21`/`0x22` write, which this model's EC does not implement. Verified: Auto ≈3.5k RPM, Max ≈8k RPM, 50% ≈6k RPM.
- **Keyboard RGB & ACPI Byte 8 Enable Flag (`KLES`)** — decoded ACPI `SSDT12` (`Method WMBH` / Method 20). The 9th byte (`BHLK[8]`) maps directly to `\_SB.PC00.LPCB.EC0.KLES` (Keyboard Lighting Enable State). Upstream tools set byte 9 while leaving byte 8 at 0, instructing the EC to extinguish the LEDs. Setting byte 8 to `1` fixes hardware illumination across both 4-zone static and dynamic effects.
- **LCD Screen Dimming Bug vs Keyboard Illumination (`Fn + F9` / `Fn + F10`)** — in upstream systemd `60-keyboard.hwdb`, generic Acer laptops map scancode `ef` to `brightnessdown` (Fn+Left screen dimming). Because the PH317-55 DMI modalias is `svnAcer:pnPredatorPH317-55:*`, it inherited this rule, causing `Fn + F10` / `Fn + F9` to dim the LCD display screen instead of the keyboard. Adding an exact DMI hwdb match mapping `ef` → `kbdillumup`, `f0` → `kbdillumdown`, and `e070` → `kbdillumdown` isolates the keyboard illumination from the display panel.
- **WMI Notify Clean Consumption** — ACPI WMI notification `0x4` (emitted by the EC on hardware backlight changes) is consumed cleanly in `acer_wmi_notify`, eliminating `dmesg` warnings and preventing synthetic duplicate toggle keypresses from conflicting with the desktop compositor.

---

## Troubleshooting

<details>
<summary><b>Keyboard RGB not changing / stuck on one effect</b></summary>

```console
sudo rmmod facer
sudo insmod /path/to/kernel/facer.ko
```

</details>

<details>
<summary><b>Module not loading</b></summary>

```console
ls /sys/bus/wmi/devices/7A4DDFE7-5B5D-40B4-8595-4408E0CC7F56/
sudo dmesg | grep -i facer
sudo apt install linux-headers-$(uname -r)
```

</details>

<details>
<summary><b>PredatorSense key not working</b></summary>

```console
systemctl --user status predator-sense-hotkey.service
pgrep -af predator-sense-hotkey
groups | grep input
sudo usermod -aG input $USER
```

A full logout/login or reboot is required after adding the user to the `input` group.

</details>

<details>
<summary><b>NVIDIA GPU page shows no data</b></summary>

```console
nvidia-smi
```

</details>

---

## Uninstall

```console
sudo ./predator-sense-installer --uninstall
```

---

## Project structure

```
predator-sense-gui/
├── kernel/        # Linux kernel modules (facer, acer-wmi-battery, acpi_ec) — DKMS-managed
├── installer/     # Rust multicall installer and background services
├── protocol/      # Shared typed GUI/helper contract
├── src/           # Rust GTK4 application
│   ├── hardware/  # Hardware backends (rgb, fan, sensors, gpu, profiles, ai, …)
│   └── ui/        # GTK4 pages, Cairo-drawn icons/chrome, and the theme
└── resources/     # Logo, model photos, stylesheet
```

---

## Credits

- Kernel module `facer` based on [acer-predator-turbo-and-rgb-keyboard-linux-module](https://github.com/JafarAkhondali/acer-predator-turbo-and-rgb-keyboard-linux-module) by [JafarAkhondali](https://github.com/JafarAkhondali) and contributors
- Kernel module `acpi_ec` by [MusiKid](https://github.com/MusiKid/acpi_ec)
- GUI built with [Rust](https://www.rust-lang.org/) + [GTK4](https://gtk.org/) + [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)

---

## License

Licensed under the **GNU General Public License v3.0** — see [LICENSE](LICENSE).

**Exception — product images:** the Acer Predator/Nitro laptop photos under `predator-sense-gui/resources/models/` are third-party product images and are **not** covered by the GPLv3 grant; all rights remain with Acer Inc. and/or the original photographers.

**This software is provided "as is", without warranty of any kind.** The authors are not responsible for any damage that may occur from using this software.
