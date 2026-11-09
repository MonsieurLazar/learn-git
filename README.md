# learn-git

A tiny Rust TUI that says hello and updates itself.

## Install

Linux and macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/MonsieurLazar/learn-git/main/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/MonsieurLazar/learn-git/main/install.ps1 | iex
```

## Use

```sh
learn-git
```

When a new release is out, the status bar shows it. Press `ctrl+u` to install it, then `ctrl+u` again to restart into the new version. `q` quits.

## Build

```sh
cargo build --release
```

Releases are built for Linux (x86_64, aarch64), macOS (x86_64, aarch64) and Windows (x86_64).
