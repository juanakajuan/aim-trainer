#!/usr/bin/env bash
set -euo pipefail
aim_room_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$aim_room_root"
cargo build --release --locked
aim_room_data="${XDG_DATA_HOME:-$HOME/.local/share}"
install -Dm755 target/release/aim-room "$HOME/.local/bin/aim-room"
install -Dm644 assets/aim-room.svg "$aim_room_data/icons/hicolor/scalable/apps/aim-room.svg"
mkdir -p "$aim_room_data/applications"
cat > "$aim_room_data/applications/aim-room.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Aim Room
Comment=Practice clicking, tracking and target switching
Exec="$HOME/.local/bin/aim-room"
Icon=aim-room
Terminal=false
Categories=Game;ActionGame;
StartupWMClass=Aim Room
EOF
if command -v update-desktop-database >/dev/null; then
    update-desktop-database "$aim_room_data/applications"
fi
printf 'Installed Aim Room. Open it from your application menu.\n'
