#!/usr/bin/env bash
# Load scenario: a spinner with colours that redraws one line 10 times per
# second. The same as spinner.ps1, for bash: see that file for the details.
#
# Usage: bash spinner.sh [frames per second]
set -uo pipefail
# shellcheck source=common.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/common.sh"

hz=${1:-10}
glyphs=(⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏)

scenario_end_text=$'\n'
scenario_start spinner
frame=0
while ! scenario_should_stop; do
  scenario_now
  # 16 to 231 are the 216 colours of the 6x6x6 cube of the 256-colour palette.
  printf '\r\e[38;5;%dm%s\e[0m \e[1mThinking\e[0m \e[2m(%d s)\e[0m\e[K' \
    $((16 + frame % 216)) "${glyphs[frame % 10]}" $((now_us / 1000000))
  frame=$((frame + 1))
  scenario_wait_frame $((1000000 / hz))
done
