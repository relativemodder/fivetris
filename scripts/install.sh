#!/usr/bin/env bash

set -euo pipefail

DO_LAUNCH=true
for arg in "$@"; do
    if [ "$arg" = "--do-not-launch" ]; then
        DO_LAUNCH=false
    fi
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"

ASSETS=("$SCRIPT_DIR/fivetris" "$SCRIPT_DIR/fivetris.desktop" "$SCRIPT_DIR/icon.png")

for asset in "${ASSETS[@]}"; do
    if [ ! -f "$asset" ]; then
        echo "Error: ${asset} not found next to install script." >&2
        echo "Make sure all release files are extracted in the same directory." >&2
        exit 1
    fi
done

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 755 "$SCRIPT_DIR/fivetris" "$BIN_DIR/fivetris"
install -m 644 "$SCRIPT_DIR/icon.png" "$ICON_DIR/fivetris.png"

sed -e "s|^Exec=.*|Exec=${BIN_DIR}/fivetris|" \
    -e "s|^Icon=.*|Icon=${ICON_DIR}/fivetris.png|" \
    "$SCRIPT_DIR/fivetris.desktop" > "$APP_DIR/fivetris.desktop"
chmod 644 "$APP_DIR/fivetris.desktop"

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$APP_DIR" &>/dev/null || true
fi

echo "Fivetris installed successfully!"
echo ""

if [[ ":$PATH:" != *":${BIN_DIR}:"* ]]; then
    echo "NOTE: ${BIN_DIR} is not in your PATH."
    echo "Add it to your shell config, e.g.:"
    echo "  echo 'export PATH=\"\${PATH}:${BIN_DIR}\"' >> ~/.bashrc"
    echo ""
fi

if [ "$DO_LAUNCH" = true ]; then
    echo "Launching Fivetris..."
    exec "$BIN_DIR/fivetris"
fi
