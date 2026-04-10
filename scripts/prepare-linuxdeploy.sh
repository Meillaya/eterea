#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  exit 0
fi

arch="$(uname -m)"
case "$arch" in
  x86_64|amd64) linuxdeploy_arch="x86_64" ;;
  aarch64|arm64) linuxdeploy_arch="aarch64" ;;
  *)
    echo "prepare-linuxdeploy: unsupported architecture '$arch'; skipping linuxdeploy override" >&2
    exit 0
    ;;
esac

tools_dir="target/.tauri"
output_path="${tools_dir}/linuxdeploy-${linuxdeploy_arch}.AppImage"
real_output_path="${tools_dir}/linuxdeploy-${linuxdeploy_arch}-real.AppImage"
launcher_source_path="${tools_dir}/linuxdeploy-launcher.c"
download_url="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${linuxdeploy_arch}.AppImage"

mkdir -p "$tools_dir"

download_with_curl() {
  curl -L --fail --silent --show-error "$download_url" -o "$real_output_path.tmp"
}

download_with_wget() {
  wget -qO "$real_output_path.tmp" "$download_url"
}

if command -v curl >/dev/null 2>&1; then
  download_with_curl
elif command -v wget >/dev/null 2>&1; then
  download_with_wget
else
  echo "prepare-linuxdeploy: neither curl nor wget is available to fetch $download_url" >&2
  exit 1
fi

mv "$real_output_path.tmp" "$real_output_path"
chmod +x "$real_output_path"

if command -v cc >/dev/null 2>&1; then
  compiler=cc
elif command -v gcc >/dev/null 2>&1; then
  compiler=gcc
elif command -v clang >/dev/null 2>&1; then
  compiler=clang
else
  echo "prepare-linuxdeploy: no C compiler found to build linuxdeploy launcher" >&2
  exit 1
fi

cat >"$launcher_source_path" <<EOF
#include <libgen.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
  char self_path[PATH_MAX];
  ssize_t len = readlink("/proc/self/exe", self_path, sizeof(self_path) - 1);
  if (len < 0) {
    perror("readlink");
    return 1;
  }

  self_path[len] = '\\0';
  char dir_path[PATH_MAX];
  strncpy(dir_path, self_path, sizeof(dir_path) - 1);
  dir_path[sizeof(dir_path) - 1] = '\\0';

  char *dir = dirname(dir_path);
  char real_path[PATH_MAX];
  snprintf(real_path, sizeof(real_path), "%s/%s", dir, "$(basename "$real_output_path")");

  setenv("NO_STRIP", "1", 0);
  execv(real_path, argv);
  perror("execv");
  return 1;
}
EOF
"$compiler" -O2 "$launcher_source_path" -o "$output_path"
chmod +x "$output_path"

echo "prepare-linuxdeploy: cached linuxdeploy wrapper at $output_path"
