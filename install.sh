#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="php-doctor"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$REPO_DIR/target/release/$BIN_NAME"
MARKER="# php-doctor alias (managed by install.sh)"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo not found. Install Rust: https://rustup.rs" >&2
    exit 1
fi

cargo build --release --manifest-path "$REPO_DIR/Cargo.toml"

case "$(basename "${SHELL:-}")" in
    zsh)  RC="$HOME/.zshrc" ;;
    bash) RC="$HOME/.bashrc" ;;
    *)    RC="$HOME/.profile" ;;
esac

ALIAS_LINE="alias $BIN_NAME=\"$BIN_PATH\""

if grep -qF "$MARKER" "$RC"; then
    tmp="$(mktemp)"
    awk -v m="$MARKER" -v a="$ALIAS_LINE" '$0==m{print;getline;print a;next}{print}' "$RC" > "$tmp"
    mv "$tmp" "$RC"
else
    printf '\n%s\n%s\n' "$MARKER" "$ALIAS_LINE" >> "$RC"
fi

echo "Done. Run: source $RC"
