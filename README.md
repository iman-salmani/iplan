<img align="left" style="vertical-align: middle" src="data/icons/hicolor/256x256/apps/ir.imansalmani.IPlan.png" alt="IPlan" width="128">

# IPlan

Your plan for improving personal life and workflow

<picture align="center">
  <source media="(prefers-color-scheme: dark)" srcset="data/screenshots/window-dark.png">
  <img alt="IPlan Window" src="data/screenshots/window.png">
</picture>

## Features

- Grouping tasks with project and list
- Timer for tasks
- Global search
- Arranging projects, lists and tasks by drag and drop

## Installation

The recommended way of installing IPlan is through Flatpak. If you don't have
Flatpak installed, you can get it from [the Flatpak website](https://flatpak.org/setup).

```bash
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install flathub ir.imansalmani.IPlan
```
<a href="https://flathub.org/apps/details/ir.imansalmani.IPlan"><img src="https://flathub.org/assets/badges/flathub-badge-en.png" alt="Download on Flathub" width="240"></a>

## Contributing

Please, see the [contribution guide](CONTRIBUTING.md) if you wish to translate.

## Build

### Gnome Builder

1. Clone the repo

```sh
git clone https://github.com/iman-salmani/iplan.git
```

2. Open project with Gnome Builder
3. Press the run button

### Manuall

1. Clone the repo and move to project directory

```sh
git clone https://github.com/iman-salmani/iplan.git && cd iplan
```

2. Install flatpak builder (flatpak-builder package available in most distributions)

- Fedora

```sh
sudo dnf install flatpak-builder
```

- Ubuntu and Debian based distributions

```sh
sudo apt install flatpak-builder
```

- Arch

```sh
sudo pacman -S flatpak-builder
```

4. Install dependencies

```sh
flatpak install runtime/org.gnome.Sdk/x86_64/43 runtime/org.freedesktop.Sdk.Extension.rust-stable/x86_64/22.08 runtime/org.gnome.Platform/x86_64/43
```

3. Build and install with flatpak builder

- System wide (Recommended)

```sh
sudo flatpak-builder --install builddir ir.imansalmani.IPlan.json --force-clean
```

- User (For testing)

```sh
flatpak-builder --install builddir ir.imansalmani.IPlan.json --force-clean --user
```

4. Run
   > App should be appear in your applications menu.

```sh
flatpak run ir.imansalmani.IPlan
```
