#!/usr/bin/env bash
# Load scenario: prints a big file as fast as the terminal takes it. The same
# as bigfile.ps1, for bash: see that file for the details. It prints with cat
# from a copy in /tmp: in WSL, reading the file from the Windows disk goes
# through 9P, and the time would be that of the file system and not that of
# the terminal.
#
# Usage: bash bigfile.sh [file]
set -uo pipefail
# shellcheck source=common.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/common.sh"

file=${1:-$scenario_dir/../../../.perf/bigfile.txt}
if [[ ! -f $file ]]; then
  echo "$file does not exist. Create it with PowerShell 7: pwsh -File tools/perf/gen-bigfile.ps1" >&2
  exit 1
fi
copy=$(mktemp "${TMPDIR:-/tmp}/meiga-bigfile.XXXXXX")
scenario_temp_files+=("$copy")
cp -- "$file" "$copy"
size=$(wc -c <"$copy")
printf -v megabytes '%d.%d' $((size / 1048576)) $((size * 10 / 1048576 % 10))

# Prints the file and sets seconds to the time that it took, such as 3.214.
print_file() {
  local start=${EPOCHREALTIME/[.,]/} end us
  cat -- "$copy"
  end=${EPOCHREALTIME/[.,]/}
  us=$((end - start))
  printf -v seconds '%d.%03d' $((us / 1000000)) $((us % 1000000 / 1000))
}

scenario_start bigfile keep-cursor
if ! $scenario_controlled; then
  print_file
  printf 'Printed %s MB in %s s.\n' "$megabytes" "$seconds"
  exit 0
fi
run=1
while :; do
  until [[ -f $run_dir/go-$run ]]; do
    if [[ -f $run_dir/stop ]]; then
      exit 0
    fi
    scenario_sleep 0.1
  done
  print_file
  # Let the terminal draw what it still has before the window ends.
  scenario_sleep 1
  printf '%s' "$seconds" >"$run_dir/finished-$run"
  printf 'Run %d: printed %s MB in %s s.\n' "$run" "$megabytes" "$seconds"
  run=$((run + 1))
done
