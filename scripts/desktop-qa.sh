#!/usr/bin/env bash
set -euo pipefail

session_name="${ETEREA_DESKTOP_QA_SESSION:-eterea-desktop-qa}"
evidence_dir="${ETEREA_DESKTOP_QA_EVIDENCE_DIR:-.omo/evidence/production-grade-project}"
log_path="$evidence_dir/desktop-tmux.log"
timing_path="$evidence_dir/desktop-first-usable.txt"
screenshot_path="$evidence_dir/live-desktop.png"
blocker_path="$evidence_dir/desktop-display-blocker.md"
wait_seconds="${ETEREA_DESKTOP_QA_WAIT_SECONDS:-12}"

mkdir -p "$evidence_dir"

has_display() {
  [[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]]
}

write_no_display_blocker() {
  cat >"$blocker_path" <<BLOCKER
# BLOCKED: live desktop screenshot unavailable

No graphical display was available for the live Dioxus desktop QA harness.
This is a no-display blocker for production approval, not a pass and not a
substitute for real desktop QA.

- captured_at_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)
- DISPLAY: ${DISPLAY:-<unset>}
- WAYLAND_DISPLAY: ${WAYLAND_DISPLAY:-<unset>}
- required_follow_up: rerun scripts/desktop-qa.sh on a host with a desktop display
BLOCKER
  echo "BLOCKED: live desktop screenshot unavailable ($blocker_path)" >&2
}

cleanup_session() {
  if tmux has-session -t "$session_name" 2>/dev/null; then
    tmux capture-pane -pt "$session_name" -S -2000 >"$log_path" || true
    tmux kill-session -t "$session_name" || true
  fi
}

if ! has_display; then
  write_no_display_blocker
  exit 3
fi

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required for desktop QA log capture" >&2
  exit 2
fi

if tmux has-session -t "$session_name" 2>/dev/null; then
  echo "tmux session already exists: $session_name" >&2
  echo "Kill it or set ETEREA_DESKTOP_QA_SESSION to a different name." >&2
  exit 2
fi

rm -f "$blocker_path" "$log_path" "$timing_path" "$screenshot_path"
trap cleanup_session EXIT

start_epoch_ns="$(date +%s%N)"

# Launch the real Dioxus desktop app. The first usable timing below is the
# elapsed wall time from launch to the live screenshot capture checkpoint; it is
# not satisfied by unit tests or a successful cargo build.
tmux new-session -d -s "$session_name" \
  "cd '$PWD' && nix develop -c cargo run -p eterea-dioxus"

sleep "$wait_seconds"
tmux capture-pane -pt "$session_name" -S -2000 >"$log_path"

if command -v gnome-screenshot >/dev/null 2>&1; then
  gnome-screenshot -f "$screenshot_path"
elif command -v import >/dev/null 2>&1; then
  import -window root "$screenshot_path"
else
  cat >"$blocker_path" <<BLOCKER
# BLOCKED: live desktop screenshot unavailable

A graphical display was present, but neither gnome-screenshot nor ImageMagick
import was available to capture the live desktop.

- captured_at_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)
- tmux_log: $log_path
- timing: $timing_path
- required_follow_up: install a screenshot tool and rerun scripts/desktop-qa.sh
BLOCKER
  echo "BLOCKED: live desktop screenshot unavailable ($blocker_path)" >&2
  exit 3
fi

end_epoch_ns="$(date +%s%N)"
python - "$start_epoch_ns" "$end_epoch_ns" "$wait_seconds" >"$timing_path" <<'PY_TIMING'
import sys
start = int(sys.argv[1])
end = int(sys.argv[2])
wait_seconds = sys.argv[3]
elapsed = (end - start) / 1_000_000_000
print(f"first_usable_screenshot_elapsed_seconds={elapsed:.3f}")
print(f"configured_wait_seconds={wait_seconds}")
print("classification=live-desktop-screenshot-checkpoint")
PY_TIMING

printf 'Live desktop QA artifacts:\n- tmux log: %s\n- first usable timing: %s\n- screenshot: %s\n' \
  "$log_path" "$timing_path" "$screenshot_path"
