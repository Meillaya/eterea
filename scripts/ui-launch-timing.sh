#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
evidence_dir="${ETEREA_T23_EVIDENCE_DIR:-$repo_root/.omo/evidence/production-grade-project}"
visual_dir="$evidence_dir/t23-visual"
session_name="${ETEREA_T23_SESSION:-eterea-ui-perf-$USER-$$}"
startup_timeout_seconds="${ETEREA_T23_STARTUP_TIMEOUT_SECONDS:-45}"
poll_seconds="${ETEREA_T23_POLL_SECONDS:-0.25}"
budget_ms="${ETEREA_T23_BUDGET_MS:-1500}"
json_path="$evidence_dir/ui-launch-timing.json"
log_path="$visual_dir/desktop-tmux.log"
run_log_path="$evidence_dir/t23-ui-harness-run.txt"
visual_report_path="$evidence_dir/t23-visual-qa-report.md"
blocker_path="$evidence_dir/t23-display-blocker.md"
interaction_waiver_path="$evidence_dir/t23-interaction-waiver.md"
full_screenshot_path="$visual_dir/live-desktop-grim.png"
app_crop_path="$visual_dir/live-eterea-region.png"
command_time_path="$visual_dir/cargo-run-time.txt"
cleanup_path="$evidence_dir/t23-cleanup.txt"
launcher_path="$visual_dir/launch-command.sh"
launched_session=false
owned_session_id=""

mkdir -p "$visual_dir"

cleanup() {
  local cleanup_started
  cleanup_started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  {
    echo "cleanup_started_utc=$cleanup_started"
    echo "session=$session_name"
    echo "owned_session_id=${owned_session_id:-}"
    echo "tmux_session_owned=$launched_session"
    if [[ "$launched_session" == true && -n "$owned_session_id" ]] && tmux has-session -t "$owned_session_id" 2>/dev/null; then
      tmux capture-pane -pt "$owned_session_id" -S -2000 >>"$log_path" 2>/dev/null || true
      tmux kill-session -t "$owned_session_id" 2>/dev/null || true
      echo "tmux_session_killed=true"
    else
      echo "tmux_session_killed=false"
    fi
    echo 'remaining_matching_processes:'
    ps -eo pid=,ppid=,stat=,comm=,args= \
      | grep -E 'eterea-ui-perf|eterea-dioxus|cargo run -p eterea-dioxus|nix develop -c cargo run' \
      | grep -v grep \
      | grep -v "$$" || echo 'none'
    echo 'remaining_matching_tmux_sessions:'
    tmux list-sessions 2>/dev/null | grep -E 'eterea-ui-perf|eterea-desktop-qa' || echo 'none'
    echo "cleanup_finished_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  } >"$cleanup_path"
}
trap cleanup EXIT INT TERM

has_display() {
  [[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]]
}

screenshot_tool() {
  if command -v grim >/dev/null 2>&1; then
    echo grim
  elif command -v gnome-screenshot >/dev/null 2>&1; then
    echo gnome-screenshot
  elif command -v import >/dev/null 2>&1; then
    echo import
  else
    echo none
  fi
}

capture_screenshot() {
  local tool="$1"
  case "$tool" in
    grim) grim "$full_screenshot_path" ;;
    gnome-screenshot) gnome-screenshot -f "$full_screenshot_path" ;;
    import) import -window root "$full_screenshot_path" ;;
    *) return 1 ;;
  esac
}

crop_app_region() {
  if [[ -s "$full_screenshot_path" ]] && command -v magick >/dev/null 2>&1; then
    magick "$full_screenshot_path" -gravity South -crop 100%x56%+0+0 +repage "$app_crop_path" || true
  fi
}

write_interaction_waiver() {
  cat >"$interaction_waiver_path" <<'WAIVER'
# T23 interaction automation waiver

Live launch/screenshot used the real Dioxus desktop app, but widget-level
automation remains blocked because this Dioxus/WebView surface has no stable
test IDs, documented DOM driver, or repo-local AT-SPI hook.
Required follow-up: add stable IDs and a desktop driver before interaction rows
can change from `blocked_no_stable_ui_driver` to `exercised`.
WAIVER
}

write_visual_report() {
  local status="$1"
  local detail="$2"
  cat >"$visual_report_path" <<REPORT
# T23 visual QA report

- status: $status
- detail: $detail
- display: DISPLAY=${DISPLAY:-<unset>}, WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-<unset>}, XDG_SESSION_TYPE=${XDG_SESSION_TYPE:-<unset>}
- artifacts: screenshot=$full_screenshot_path; app_region_crop=$app_crop_path; tmux_log=$log_path; interaction_waiver=$interaction_waiver_path
- scope: T23 launch/timing only; T28 owns deep visual/accessibility parity.
REPORT
}

write_json() {
  local first_ms="$1"
  local screenshot_ms="$2"
  local delay_ms="$3"
  local pass_value="$4"
  local status="$5"
  local blocker="$6"
  FIRST_MS="$first_ms" SCREENSHOT_MS="$screenshot_ms" DELAY_MS="$delay_ms" PASS_VALUE="$pass_value" STATUS="$status" BLOCKER="$blocker" \
  JSON_PATH="$json_path" LOG_PATH="$log_path" FULL_SCREENSHOT="$full_screenshot_path" \
  APP_CROP="$app_crop_path" CLEANUP_PATH="$cleanup_path" COMMAND_TIME="$command_time_path" \
  INTERACTION_WAIVER="$interaction_waiver_path" BUDGET_MS="$budget_ms" SESSION_NAME="$session_name" \
  STARTUP_TIMEOUT="$startup_timeout_seconds" POLL_SECONDS="$poll_seconds" REPO_ROOT="$repo_root" \
  python3 - <<'PY_JSON'
import json, os
from pathlib import Path
first_raw, screenshot_raw = os.environ["FIRST_MS"], os.environ["SCREENSHOT_MS"]
first_ms = None if first_raw == "null" else int(first_raw)
screenshot_ms = None if screenshot_raw == "null" else int(screenshot_raw)
pass_value = os.environ["PASS_VALUE"] == "true"
payload = {
    "schema_version": 1,
    "captured_at_utc": __import__("datetime").datetime.now(__import__("datetime").timezone.utc).isoformat(),
    "scenario": "T23 live Dioxus desktop launch timing via tmux and screenshot checkpoint",
    "command": "timeout 180s scripts/ui-launch-timing.sh",
    "launch_command": "tmux new-session ... /usr/bin/time -f %e nix develop -c cargo run -p eterea-dioxus",
    "first_usable_shell_ms": first_ms,
    "first_usable_basis": "elapsed wall time from harness start to startup log readiness checkpoint, before screenshot stabilization delay",
    "post_ready_screenshot_delay_ms": int(os.environ["DELAY_MS"]),
    "screenshot_elapsed_ms": screenshot_ms,
    "budget_ms": int(os.environ["BUDGET_MS"]),
    "display": {"DISPLAY": os.environ.get("DISPLAY"), "WAYLAND_DISPLAY": os.environ.get("WAYLAND_DISPLAY"), "XDG_SESSION_TYPE": os.environ.get("XDG_SESSION_TYPE"), "screenshot_tool": os.environ.get("SCREENSHOT_TOOL", "none"), "available": bool(os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"))},
    "pass": pass_value,
    "status": os.environ["STATUS"],
    "blocker": os.environ["BLOCKER"] or None,
    "release_evidence": False,
    "sample_count": 1,
    "classification": "cold_process_start_dev_guardrail",
    "interaction_results": [{"name": name, "status": "blocked_no_stable_ui_driver", "artifact": os.environ["INTERACTION_WAIVER"]} for name in ["search", "author_filter", "topic_filter", "layout_switch", "import_preview", "settings_density_tone", "remote_media_opt_in_off"]],
    "artifacts": {"tmux_log": os.environ["LOG_PATH"], "screenshot": os.environ["FULL_SCREENSHOT"], "app_region_crop": os.environ["APP_CROP"], "command_time": os.environ["COMMAND_TIME"], "cleanup": os.environ["CLEANUP_PATH"], "interaction_waiver": os.environ["INTERACTION_WAIVER"]},
    "environment": {"repo_root": os.environ["REPO_ROOT"], "tmux_session": os.environ["SESSION_NAME"], "startup_timeout_seconds": float(os.environ["STARTUP_TIMEOUT"])},
}
Path(os.environ["JSON_PATH"]).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY_JSON
}

main() {
  : >"$run_log_path"
  echo "started_at_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)" | tee -a "$run_log_path"
  echo "session=$session_name" | tee -a "$run_log_path"
  echo "repo=$repo_root" | tee -a "$run_log_path"
  write_interaction_waiver

  if [[ ! "$session_name" =~ ^[-_A-Za-z0-9]+$ ]]; then
    write_visual_report BLOCKED "unsafe tmux session name rejected before launch"
    write_json null null 0 false blocked_invalid_session_name "unsafe tmux session name: contains tmux target-special character (: . $ ;)"
    echo "BLOCKED: unsafe tmux session name rejected before launch: $session_name" | tee -a "$run_log_path"
    return 0
  fi

  if ! has_display; then
    cat >"$blocker_path" <<BLOCKER
# BLOCKED: live UI proof unavailable

No graphical display was available; release approval remains open.
DISPLAY=${DISPLAY:-<unset>} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-<unset>}
BLOCKER
    write_visual_report BLOCKED "no graphical display"
    write_json null null 0 false blocked_no_display "$blocker_path"
    echo "BLOCKED: no display; wrote $json_path" | tee -a "$run_log_path"
    return 0
  fi

  if ! command -v tmux >/dev/null 2>&1; then
    write_visual_report BLOCKED "tmux missing"
    write_json null null 0 false blocked_missing_tmux "tmux missing"
    echo "BLOCKED: tmux missing" | tee -a "$run_log_path"
    return 0
  fi

  local tool
  tool="$(screenshot_tool)"
  export SCREENSHOT_TOOL="$tool"
  echo "screenshot_tool=$tool" | tee -a "$run_log_path"
  if [[ "$tool" == none ]]; then
    write_visual_report BLOCKED "no screenshot tool"
    write_json null null 0 false blocked_no_screenshot_tool "no screenshot tool"
    echo "BLOCKED: no screenshot tool" | tee -a "$run_log_path"
    return 0
  fi

  if tmux has-session -t "$session_name" 2>/dev/null; then
    echo "session already exists: $session_name" | tee -a "$run_log_path"
    write_json null null 0 false blocked_session_exists "tmux session already exists"
    return 0
  fi

  rm -f "$log_path" "$full_screenshot_path" "$app_crop_path" "$command_time_path" "$blocker_path"
  cat >"$launcher_path" <<LAUNCH
#!/usr/bin/env bash
cd "$repo_root"
if [[ -x /usr/bin/time ]]; then exec /usr/bin/time -f 'cargo_run_wall_seconds=%e' -o "$command_time_path" nix develop -c cargo run -p eterea-dioxus; fi
echo 'time_binary_missing=/usr/bin/time' >"$command_time_path"
exec nix develop -c cargo run -p eterea-dioxus
LAUNCH
  chmod +x "$launcher_path"
  local start_ns now_ns first_usable_ms screenshot_ms delay_seconds delay_ms deadline_ns ready=false
  start_ns="$(date +%s%N)"
  deadline_ns=$((start_ns + startup_timeout_seconds * 1000000000))
  if tmux new-session -d -s "$session_name" "$launcher_path >>'$log_path' 2>&1"; then
    launched_session=true
    owned_session_id="$(tmux display-message -p -t "$session_name" '#{session_id}' 2>/dev/null || true)"
    echo "tmux_launch_status=0" | tee -a "$run_log_path"
  else
    echo "tmux_launch_status=1" | tee -a "$run_log_path"
    write_json null null 0 false blocked_tmux_launch_failed "tmux new-session failed"
    return 0
  fi

  while true; do
    tmux capture-pane -pt "$session_name" -S -2000 >>"$log_path" 2>/dev/null || true
    if grep -Eq 'loaded stats snapshot|loaded bookmark page|Opening database|opening database' "$log_path"; then
      ready=true
      break
    fi
    now_ns="$(date +%s%N)"
    if (( now_ns >= deadline_ns )); then
      break
    fi
    sleep "$poll_seconds"
  done

  if [[ "$ready" != true ]]; then
    write_visual_report BLOCKED "startup log checkpoint not reached before timeout"
    write_json null null 0 false blocked_startup_timeout "startup checkpoint timeout"
    echo "BLOCKED: startup checkpoint timeout" | tee -a "$run_log_path"
    return 0
  fi

  now_ns="$(date +%s%N)"
  first_usable_ms=$(((now_ns - start_ns) / 1000000))
  delay_seconds="${ETEREA_T23_POST_READY_DELAY_SECONDS:-8}"
  delay_ms="$(python3 - <<PY_MS
print(int(float("$delay_seconds") * 1000))
PY_MS
)"
  sleep "$delay_seconds"

  if capture_screenshot "$tool"; then
    now_ns="$(date +%s%N)"
    screenshot_ms=$(((now_ns - start_ns) / 1000000))
    crop_app_region
    local pass=false status=failed_budget_or_interaction_waiver
    if (( first_usable_ms <= budget_ms )); then
      status="failed_interaction_waiver"
    fi
    write_visual_report PASS_WITH_RELEASE_WAIVERS "screenshot captured; JSON pass remains false until budget and interaction waiver close"
    write_json "$first_usable_ms" "$screenshot_ms" "$delay_ms" "$pass" "$status" "interaction automation waiver remains open"
    echo "first_usable_shell_ms=$first_usable_ms" | tee -a "$run_log_path"
    echo "screenshot_elapsed_ms=$screenshot_ms" | tee -a "$run_log_path"
    echo "json=$json_path" | tee -a "$run_log_path"
  else
    write_visual_report BLOCKED "screenshot command failed"
    write_json null null 0 false blocked_screenshot_failed "screenshot command failed"
    echo "BLOCKED: screenshot failed" | tee -a "$run_log_path"
  fi
}

main "$@"
