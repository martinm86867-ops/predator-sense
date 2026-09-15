# Predator Sense for Linux

<p align="center">
  <img src="predator-sense-gui/resources/logo-256.png" width="120" alt="Predator Sense Logo">
</p>

<p align="center">
  <b>Unofficial Linux kernel module and GUI for Acer gaming-laptop hardware control</b><br>
  <i>RGB Keyboard Backlighting &bull; Turbo Mode &bull; Temperature Monitoring &bull; Performance Profiles</i>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/GTK-4-blue?logo=gtk" alt="GTK4">
  <img src="https://img.shields.io/badge/License-GPL--3.0-green" alt="License">
  <img src="https://img.shields.io/badge/Platform-Linux-yellow?logo=linux" alt="Linux">
</p>

---

## Disclaimer

> **Use at your own risk!** This is an **unofficial** project; Acer was not involved in its development. The kernel module was built by reverse-engineering the official Windows PredatorSense application and drives low-level WMI/ACPI interfaces that have not been tested on every laptop series. The authors are not responsible for any damage to your hardware.
>
> All trademarks, product names, and logos (Acer, Predator, PredatorSense, Helios, Nitro, etc.) belong to their respective owners. This project is not affiliated with, endorsed by, or sponsored by Acer Inc.

---

## About

Unofficial Linux kernel module for Acer gaming-laptop RGB keyboard backlighting and Turbo mode (Acer Predator, Helios, Nitro), plus a full desktop application built with Rust and GTK4.

Based on the [acer-predator-turbo-and-rgb-keyboard-linux-module](https://github.com/JafarAkhondali/acer-predator-turbo-and-rgb-keyboard-linux-module) project by [JafarAkhondali](https://github.com/JafarAkhondali) and contributors.

---

## Features

| Feature | Description |
|---------|-------------|
| **Dashboard** | Live instrument cluster + complete system specs (CPU, GPU, RAM, storage, network, OS) |
| **Temperatures** | Live gauges for CPU, GPU, system, NVMe, WiFi and RAM |
| **Usage** | CPU / GPU / Memory / Storage with top processes |
| **Network** | Real-time download/upload graphs with peak tracking and auto interface detection |
| **RGB keyboard** | Static per-zone (4 zones) and dynamic effects (Breathing, Neon, Wave, Shifting, Zoom) over WMI, or natively over I2C/USB-HID on newer hardware |
| **RGB cover logo** | Power, solid color, brightness, Breathing/Neon for the display-lid emblem (runtime-detected) |
| **Performance profiles** | Quiet / Balanced / Performance / Turbo, plus a battery-only Eco tier |
| **Fan control** | Live RPM with animated fans, CoolBoost toggle, Auto/Max modes |
| **Cooling hub** | Performance modes, fan control, and the GPU dashboard in one tabbed page |
| **Battery** | Charge stats, cycles, health, and 80% charge limit for longevity |
| **GPU dashboard** | NVIDIA metrics: temperature, utilization, VRAM, clocks, power draw, PCIe info, and a power-limit (TGP) slider |
| **Graphs** | Detailed CPU/GPU history charts with min/max tracking |
| **AI assistant** 🧪 | Opt-in, local (Ollama) assistant with a fixed, already-validated action set |
| **System tray + hotkey** | Minimize to tray; the PredatorSense key opens the app |
| **Auto capability detection** | Unsupported features are shown as "not available on this model" instead of erroring |
| **Hardware report** | Detected capabilities, per-control confidence, and the reverse-engineering notes for this machine |
| **DKMS** | Kernel modules rebuild automatically across kernel upgrades |
| **Internationalization** | English, Portuguese, Spanish, Chinese, Japanese, Russian, German, Italian, Turkish |

---

## Installation

### Prebuilt installer (fastest)

```console
curl --fail --location https://github.com/cleyton1986/predator-sense/releases/latest/download/predator-sense-installer --output predator-sense-installer
chmod +x predator-sense-installer
sudo ./predator-sense-installer --install
```

The installer, privileged helper, hotkey listener, and tray service are all provided by the same Rust multicall binary.

### Build from source

**Prerequisites** (Debian/Ubuntu shown; adjust for other distros):

```console
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config build-essential \
    gcc make dkms curl tar linux-headers-$(uname -r)
```

Rust (if not installed):

```console
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Build & install:**

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

1. Open **Lighting** in the sidebar
2. Choose **Static** (per-zone colors) or **Dynamic** (effects)
3. Adjust colors/effects and speed
4. Click **Apply**

### GPU Dashboard

Real-time NVIDIA monitoring: temperature, utilization, VRAM, power draw, clocks, P-State, PCIe link info, and VBIOS version.

### AI Assistant (beta)

Opt-in, local assistant powered by [Ollama](https://ollama.com):

1. Install Ollama separately ([official instructions](https://ollama.com/download/linux))
2. Download a model from the built-in model manager (`smollm2:1.7b` or larger)
3. Enable the assistant in Settings and choose **Auto-apply** or **Always confirm**

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
| **PH317-55** | ✅ | ✅ | ✅ | 🟡 | ✅ | - | ✅ |
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

> If your model is not listed, it may still work — the kernel module detects compatible WMI interfaces automatically. Please open an issue mentioning your model so this table can be updated.

### PH317-55 turbo and fan control

The **PH317-55** (Predator Helios 300 2021) is this fork's primary target. Its firmware exposes the full gaming WMI interface (GUID `7A4DDFE7-5B5D-40B4-8595-4408E0CC7F56`, object `"BG"` → `WMBG` → `WSMI` → EC port `0xD0`), but upstream `facer.c` had no DMI quirk for it, so turbo and fan control were never enabled.

- **Turbo button** — added `quirk_acer_predator_ph317_55` with `turbo = 1`. The physical turbo key now toggles OC mode (`0x205`/`0x207`), turbo fan (method 14) and the turbo LED over the gaming WMI interface. Verified: fans ramp from ~3.5k to ~8k RPM.
- **Fan control** — added `pwm = 1` so Auto/Max and per-fan manual % route through the WMI-backed hwmon PWM path (methods 14–17) instead of the raw EC `0x21`/`0x22` write, which this model's EC does not implement (issue #1). Verified: Auto ≈3.5k RPM, Max ≈8k RPM, 50% ≈6k RPM.

---

## Troubleshooting

<details>
<summary><b>Keyboard RGB not changing / stuck on one effect</b></summary>

Reload the kernel module:

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

Verify the NVIDIA driver works:

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
│   └── ui/        # GTK4 pages and custom widgets
└── resources/     # Icons, model photos, theme
```

---

## Credits

- Kernel module `facer` based on [acer-predator-turbo-and-rgb-keyboard-linux-module](https://github.com/JafarAkhondali/acer-predator-turbo-and-rgb-keyboard-linux-module) by [JafarAkhondali](https://github.com/JafarAkhondali) and contributors
- Kernel module `acpi_ec` by [MusiKid](https://github.com/MusiKid/acpi_ec)
- GUI built with [Rust](https://www.rust-lang.org/) + [GTK4](https://gtk.org/) + [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)

---

## License

This project is licensed under the **GNU General Public License v3.0** — see the [LICENSE](LICENSE) file for details.

**Exception — product images:** the Acer Predator/Nitro laptop photos under `predator-sense-gui/resources/models/` are third-party product images and are **not** covered by the GPLv3 grant; all rights in those images remain with Acer Inc. and/or the original photographers.

**This software is provided "as is", without warranty of any kind.** The authors are not responsible for any damage that may occur from using this software.
