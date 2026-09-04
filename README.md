# Buffer-Pie

<div align="center">
<img src="/readme/OG.png">

**Interactive Pi menu, now for your favourite Operating System!**

[Download](https://github.com/ppmpreetham/bufferpie/releases) · [Discord](https://www.google.com/search?q=https://discord.gg/rickroll)

</div>

## Building from source

### Linux Dependencies

If you are building on Linux, you might encounter missing library errors (like `gdk-3.0`, `xkbcommon-x11`, or `xdo`). This happens when development packages are missing on your system.

Please install the required dependencies using your package manager:

**Ubuntu / Debian / Mint:**
```bash
sudo apt update
sudo apt install libgtk-3-dev libxdo-dev libxkbcommon-x11-dev
```

**Fedora / RHEL / CentOS:**
```bash
sudo dnf install gtk3-devel libxdo-devel libxkbcommon-x11-devel
```

**Arch Linux / Manjaro:**
```bash
sudo pacman -S gtk3 xdotool libxkbcommon-x11
```

**openSUSE:**
```bash
sudo zypper install gtk3-devel xdotool-devel libxkbcommon-x11-devel
```
