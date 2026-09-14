#!/usr/bin/env bash
set -euo pipefail
aim_room_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$aim_room_root"
if [[ ! -x target/release/aim-room ]]; then
    cargo build --release --locked
fi
exec "$aim_room_root/target/release/aim-room" "$@"
