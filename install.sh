#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release --locked
token_speed_install_dir="${TOKEN_SPEED_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$token_speed_install_dir"
# Replace an earlier Python tool's launcher symlink, not its target.
if [[ -L "$token_speed_install_dir/token-speed" ]]; then
    rm "$token_speed_install_dir/token-speed"
fi
install -m 755 target/release/token-speed "$token_speed_install_dir/token-speed"
printf 'Installed %s/token-speed\nRun: token-speed --today\n' "$token_speed_install_dir"
