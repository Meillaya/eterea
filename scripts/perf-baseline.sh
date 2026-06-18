#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage:
  scripts/perf-baseline.sh
  scripts/perf-baseline.sh --stress <count>
  scripts/perf-baseline.sh --help

Run from the repository root inside the Nix development shell:
  nix develop -c scripts/perf-baseline.sh
  nix develop -c scripts/perf-baseline.sh --stress <count>

Runs Eterea's deterministic generated-archive performance harness.

Default mode runs the release-blocking service performance baseline, including
10k budgets from src/app/tests/performance_baseline.rs.

Stress mode runs the ignored stress-lab test with ETEREA_STRESS_COUNT=<count>.
Supported staged counts are read from fixtures/perf/stress-tiers.txt. Stress
output is written to
target/eterea/perf/stress-lab/performance_stress_lab_<count>.json with
release_evidence=false. Stress targets are lab artifacts until hardware,
cold/warm run classification, file-backed SQLite/WAL evidence, and budgets are
approved.
USAGE
}

stress_tiers_file="fixtures/perf/stress-tiers.txt"

require_stress_tiers_file() {
  if [[ ! -r "$stress_tiers_file" ]]; then
    echo "missing stress tiers file: $stress_tiers_file" >&2
    exit 2
  fi
}

supported_stress_counts() {
  require_stress_tiers_file
  tr '\n' ' ' <"$stress_tiers_file" | sed 's/[[:space:]]*$//'
}

is_supported_stress_count() {
  local candidate="$1"
  require_stress_tiers_file
  grep -qx "$candidate" "$stress_tiers_file"
}

write_environment_report() {
  local report_dir="target/eterea/perf"
  mkdir -p "$report_dir"
  cat >"$report_dir/perf_environment.json" <<EOF_ENV
{
  "generated_at_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "os": "$(uname -s | sed 's/"/\\"/g')",
  "kernel": "$(uname -r | sed 's/"/\\"/g')",
  "machine": "$(uname -m | sed 's/"/\\"/g')",
  "rustc": "$(rustc --version | sed 's/"/\\"/g')",
  "cargo_profile": "test"
}
EOF_ENV
}

if [[ $# -eq 0 ]]; then
  export ETEREA_PERF_GENERATED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  export ETEREA_PERF_KERNEL="$(uname -r)"
  export ETEREA_PERF_RUSTC="$(rustc --version)"
  RUST_TEST_THREADS=1 cargo test -p eterea-app --test performance_baseline -- --nocapture
  write_environment_report
  exit 0
fi

case "${1:-}" in
  --stress)
    if [[ $# -ne 2 ]]; then
      usage >&2
      exit 2
    fi
    if ! [[ "$2" =~ ^[1-9][0-9]*$ ]]; then
      echo "stress count must be a positive integer" >&2
      exit 2
    fi
    if ! is_supported_stress_count "$2"; then
      echo "unsupported stress count: $2 (use $(supported_stress_counts))" >&2
      exit 2
    fi
    export ETEREA_PERF_GENERATED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    export ETEREA_PERF_KERNEL="$(uname -r)"
    export ETEREA_PERF_RUSTC="$(rustc --version)"
    RUST_TEST_THREADS=1 ETEREA_STRESS_COUNT="$2" cargo test -p eterea-app --test performance_baseline \
      stress_archive_report_for_configured_count -- --ignored --nocapture
    write_environment_report
    ;;
  --help|-h)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
