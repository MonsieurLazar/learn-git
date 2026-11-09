#!/bin/sh

set -eu

REPO="${LEARN_GIT_INSTALL_REPO:-MonsieurLazar/learn-git}"
BINARY="learn-git"
INSTALL_DIR="${LEARN_GIT_INSTALL_DIR:-$HOME/.local/bin}"
VERSION=""
tmp=""

cleanup() {
    [ -n "$tmp" ] && rm -rf "$tmp"
}

trap cleanup EXIT INT TERM

usage() {
    cat <<'EOF'
Usage: install.sh [--version <tag>] [--dir <path>]

  --version <tag>  Install a specific release, for example v0.1.0
  --dir <path>     Install directory (default: ~/.local/bin)
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'unknown argument: %s\n' "$1" >&2
            exit 1
            ;;
    esac
done

need() {
    command -v "$1" >/dev/null 2>&1 || {
        printf 'missing required command: %s\n' "$1" >&2
        exit 1
    }
}

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" "$1"
    else
        printf 'missing required command: curl or wget\n' >&2
        exit 1
    fi
}

need uname
need mktemp
need tar
need chmod
need mkdir

case "$(uname -s)" in
    Linux) OS="unknown-linux-gnu" ;;
    Darwin) OS="apple-darwin" ;;
    *)
        printf 'unsupported operating system: %s\n' "$(uname -s)" >&2
        exit 1
        ;;
esac

case "$(uname -m)" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *)
        printf 'unsupported architecture: %s\n' "$(uname -m)" >&2
        exit 1
        ;;
esac

TARGET="${ARCH}-${OS}"
tmp=$(mktemp -d)

if [ -z "$VERSION" ]; then
    download "https://api.github.com/repos/$REPO/releases/latest" "$tmp/release.json"
    VERSION=$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$tmp/release.json" | head -n 1)
fi

[ -n "$VERSION" ] || {
    printf 'could not determine the latest release\n' >&2
    exit 1
}

ASSET="${BINARY}-${VERSION}-${TARGET}.tar.gz"
BASE="https://github.com/$REPO/releases/download/$VERSION"

download "$BASE/$ASSET" "$tmp/$ASSET"
download "$BASE/SHA256SUMS" "$tmp/SHA256SUMS"

EXPECTED=$(grep "  $ASSET$" "$tmp/SHA256SUMS" | awk '{print $1}')
[ -n "$EXPECTED" ] || {
    printf 'no checksum for %s\n' "$ASSET" >&2
    exit 1
}

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "$tmp/$ASSET" | awk '{print $1}')
else
    ACTUAL=$(shasum -a 256 "$tmp/$ASSET" | awk '{print $1}')
fi

[ "$EXPECTED" = "$ACTUAL" ] || {
    printf 'checksum mismatch for %s\n' "$ASSET" >&2
    exit 1
}

tar -xzf "$tmp/$ASSET" -C "$tmp"
mkdir -p "$INSTALL_DIR"

if command -v install >/dev/null 2>&1; then
    install "$tmp/$BINARY" "$INSTALL_DIR/$BINARY"
else
    cp "$tmp/$BINARY" "$INSTALL_DIR/$BINARY"
    chmod 755 "$INSTALL_DIR/$BINARY"
fi

printf 'installed %s %s to %s/%s\n' "$BINARY" "$VERSION" "$INSTALL_DIR" "$BINARY"

case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) printf 'warning: %s is not on your PATH\n' "$INSTALL_DIR" >&2 ;;
esac
