#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION="zrs-smoke-$$"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/zrs-smoke.XXXXXX")"
DEFAULT_ZJSTATUS_WASM="$HOME/.config/zellij/plugins/zjstatus.wasm"
if [[ ! -f "$DEFAULT_ZJSTATUS_WASM" && -f "$HOME/.config/zellij/plugins/zjstatus-v0.23.0.wasm" ]]; then
  DEFAULT_ZJSTATUS_WASM="$HOME/.config/zellij/plugins/zjstatus-v0.23.0.wasm"
fi
ZJSTATUS_WASM="${ZJSTATUS_WASM:-$DEFAULT_ZJSTATUS_WASM}"
ZELLIJ_BIN="${ZELLIJ_BIN:-/usr/local/bin/zellij}"
PLUGIN_WASM="$ROOT_DIR/target/zellij-resource-status.wasm"
LAYOUT="$TMP_DIR/layout.kdl"
CONFIG="$TMP_DIR/config.kdl"
LOCKED_DUMP="$TMP_DIR/locked.txt"
NORMAL_DUMP="$TMP_DIR/normal.txt"
LAUNCH_LOG="$TMP_DIR/launch.log"
PERMISSIONS_CACHE="${ZELLIJ_PERMISSIONS_CACHE:-$HOME/Library/Caches/org.Zellij-Contributors.Zellij/permissions.kdl}"
PERMISSIONS_BACKUP="$TMP_DIR/permissions.kdl.bak"
PERMISSIONS_EXISTED=0
LAUNCH_MODE="${ZRS_SMOKE_LAUNCH_MODE:-script}"
RESOURCE_PATTERN='[0-9]+% .*([0-9]+\.[0-9]+/[0-9]+\.[0-9]+G)'

cleanup() {
  "$ZELLIJ_BIN" kill-session "$SESSION" >/dev/null 2>&1 || true
  "$ZELLIJ_BIN" delete-session "$SESSION" >/dev/null 2>&1 || true
  if [[ "$PERMISSIONS_EXISTED" == "1" ]]; then
    cp "$PERMISSIONS_BACKUP" "$PERMISSIONS_CACHE"
  else
    rm -f "$PERMISSIONS_CACHE"
  fi
  if [[ "${ZRS_SMOKE_KEEP_TMP:-0}" != "1" ]]; then
    rm -rf "$TMP_DIR"
  else
    echo "kept smoke artifacts in $TMP_DIR" >&2
  fi
}
trap cleanup EXIT

if [[ ! -f "$ZJSTATUS_WASM" ]]; then
  echo "Set ZJSTATUS_WASM to the zjstatus .wasm path; default missing: $ZJSTATUS_WASM" >&2
  exit 1
fi
if [[ ! -x "$ZELLIJ_BIN" ]]; then
  echo "ZELLIJ_BIN is not executable: $ZELLIJ_BIN" >&2
  exit 1
fi
mkdir -p "$(dirname "$PERMISSIONS_CACHE")"
if [[ -f "$PERMISSIONS_CACHE" ]]; then
  cp "$PERMISSIONS_CACHE" "$PERMISSIONS_BACKUP"
  PERMISSIONS_EXISTED=1
fi
python3 - "$PERMISSIONS_CACHE" "file:$PLUGIN_WASM" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
key = sys.argv[2]
replacement = f'"{key}" {{\n    RunCommands\n    MessageAndLaunchOtherPlugins\n}}\n'
lines = path.read_text().splitlines(keepends=True) if path.exists() else []
out = []
i = 0
while i < len(lines):
    if lines[i].strip() == f'"{key}" {{':
        i += 1
        while i < len(lines) and lines[i].strip() != "}":
            i += 1
        if i < len(lines):
            i += 1
        continue
    out.append(lines[i])
    i += 1
if out and not out[-1].endswith("\n"):
    out[-1] += "\n"
out.append(replacement)
path.write_text("".join(out))
PY


"$ROOT_DIR/scripts/build-wasm.sh" >/dev/null

cat >"$CONFIG" <<KDL
default_mode "locked"
session_serialization false
pane_frames false
show_startup_tips false
show_release_notes false
KDL

cat >"$LAYOUT" <<KDL
layout {
    pane command="sleep" {
        args "60"
    }
    pane size=1 borderless=true {
        plugin location="file:$ZJSTATUS_WASM" {
            format_left "{mode}{tabs}"
            format_center ""
            format_right "{pipe_resources}#[fg=#1A1B26,bg=#7AA2F7,bold]  #[fg=#A9B1D6,bg=#3B4261,bold] {session} "
            format_space "#[bg=#1A1B26]"

            border_enabled "false"
            hide_frame_for_single_pane "false"

            mode_locked        "#[bg=#F7768E] #[bg=#1A1B26] "
            mode_normal        "#[bg=#9ECE6A] #[bg=#1A1B26] "
            mode_pane          "#[bg=#7AA2F7] #[bg=#1A1B26] "
            mode_tab           "#[bg=#7AA2F7] #[bg=#1A1B26] "
            mode_resize        "#[bg=#E0AF68] #[bg=#1A1B26] "
            mode_move          "#[bg=#E0AF68] #[bg=#1A1B26] "
            mode_scroll        "#[bg=#BB9AF7] #[bg=#1A1B26] "
            mode_search        "#[bg=#BB9AF7] #[bg=#1A1B26] "
            mode_enter_search  "#[bg=#BB9AF7] #[bg=#1A1B26] "
            mode_rename_tab    "#[bg=#E0AF68] #[bg=#1A1B26] "
            mode_rename_pane   "#[bg=#E0AF68] #[bg=#1A1B26] "
            mode_session       "#[bg=#7DCFFF] #[bg=#1A1B26] "
            mode_prompt        "#[bg=#7DCFFF] #[bg=#1A1B26] "
            mode_tmux          "#[bg=#E0AF68] #[bg=#1A1B26] "
            mode_default_to_mode "normal"

            tab_normal            "#[fg=#565F89] {index}:#[fg=#A9B1D6]{name}{sync_indicator}{fullscreen_indicator}{floating_indicator} "
            tab_active            "#[fg=#1A1B26,bg=#7AA2F7,bold] ✓ {name}{sync_indicator}{fullscreen_indicator}{floating_indicator} #[fg=#7AA2F7,bg=#1A1B26]"
            tab_rename            "#[fg=#1A1B26,bg=#E0AF68,bold] {index}: {name}{sync_indicator}{fullscreen_indicator}{floating_indicator} #[fg=#E0AF68,bg=#1A1B26]"
            tab_separator         "#[fg=#565F89,bg=#1A1B26] "
            tab_sync_indicator       " [sync]"
            tab_fullscreen_indicator " [full]"
            tab_floating_indicator   " [float]"

            pipe_resources_format     "{output}"
            pipe_resources_rendermode "dynamic"
        }
    }
    pane borderless=true {
        plugin location="file:$PLUGIN_WASM" {
            pipe_name "pipe_resources"
            interval_secs "10"
        }
    }
}
KDL

case "$LAUNCH_MODE" in
  direct)
    env -u ZELLIJ -u ZELLIJ_SESSION_NAME "$ZELLIJ_BIN" --config "$CONFIG" --session "$SESSION" --new-session-with-layout "$LAYOUT" >"$LAUNCH_LOG" 2>&1 &
    ZELLIJ_PID=$!
    ;;
  script)
    (sleep 120) | script -q "$LAUNCH_LOG" env -u ZELLIJ -u ZELLIJ_SESSION_NAME "$ZELLIJ_BIN" --config "$CONFIG" --session "$SESSION" --new-session-with-layout "$LAYOUT" >/dev/null 2>&1 &
    ZELLIJ_PID=$!
    ;;
  *)
    echo "Unsupported ZRS_SMOKE_LAUNCH_MODE: $LAUNCH_MODE" >&2
    exit 1
    ;;
esac

for _ in {1..20}; do
  if "$ZELLIJ_BIN" list-sessions --short 2>/dev/null | grep -qx "$SESSION"; then
    break
  fi
  sleep 0.5
done
if ! "$ZELLIJ_BIN" list-sessions --short 2>/dev/null | grep -qx "$SESSION"; then
  echo "Zellij smoke session did not start" >&2
  cat "$LAUNCH_LOG" >&2
  exit 1
fi

sleep 12
cp "$LAUNCH_LOG" "$LOCKED_DUMP"
if ! grep -Eq "$RESOURCE_PATTERN" "$LOCKED_DUMP"; then
  echo "resource segments were not visible in locked mode" >&2
  cat "$LOCKED_DUMP" >&2
  exit 1
fi
LOCKED_LOG_BYTES="$(wc -c <"$LAUNCH_LOG" | tr -d ' ')"

"$ZELLIJ_BIN" --session "$SESSION" action switch-mode normal
sleep 2
tail -c +"$((LOCKED_LOG_BYTES + 1))" "$LAUNCH_LOG" >"$NORMAL_DUMP"
if ! grep -Eq "$RESOURCE_PATTERN" "$NORMAL_DUMP"; then
  echo "resource segments were not visible after switching to normal mode" >&2
  cat "$NORMAL_DUMP" >&2
  exit 1
fi

kill "$ZELLIJ_PID" >/dev/null 2>&1 || true
trap - EXIT
cleanup
if "$ZELLIJ_BIN" list-sessions --short 2>/dev/null | grep -qx "$SESSION"; then
  echo "smoke session still exists after cleanup: $SESSION" >&2
  exit 1
fi

echo "smoke cleanup confirmed for session $SESSION"
echo "locked-mode smoke passed for session $SESSION"
