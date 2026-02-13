#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="$SCRIPT_DIR/build/install/fm/bin"
SYMLINK="$HOME/.local/bin/fm"

echo "Building fm..."
"$SCRIPT_DIR/gradlew" -p "$SCRIPT_DIR" installDist

mkdir -p "$HOME/.local/bin"

if [ -L "$SYMLINK" ] || [ -e "$SYMLINK" ]; then
    rm "$SYMLINK"
fi

ln -s "$INSTALL_DIR/fm" "$SYMLINK"
echo "Symlinked $SYMLINK -> $INSTALL_DIR/fm"

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$HOME/.local/bin"; then
    echo ""
    echo "WARNING: ~/.local/bin is not on your PATH."
    echo "Add this to your ~/.zshrc (or ~/.bashrc):"
    echo ""
    echo '  export PATH="$HOME/.local/bin:$PATH"'
    echo ""
fi

echo "Done! Run 'fm start' to get started."
