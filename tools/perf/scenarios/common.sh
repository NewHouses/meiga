# shellcheck shell=bash
# Shared by the bash load scenarios (spinner.sh, ai-stream.sh and bigfile.sh),
# which source it: terminal set-up, the files that coordinate a scenario with
# run-scenario.ps1, and frame timing. Scenario.psm1 does the same for the
# PowerShell scenarios.
#
# Under run-scenario.ps1, the run folder (.perf/run/, or the folder in
# $MEIGA_PERF_RUN) contains a file named "controller". A scenario then creates
# "started-<scenario>-bash-<process id>" when it starts, deletes it when it
# ends, and ends when the file "stop" appears. Without "controller", it runs
# until Ctrl+C.
#
# Nothing here starts a process once the scenario runs: the clock is
# $EPOCHREALTIME and the sleeps are read timeouts.

if ((BASH_VERSINFO[0] < 5)); then
  echo "The load scenarios need bash 5 or newer; this is bash $BASH_VERSION." >&2
  exit 1
fi

# The escape character, for the scripts that source this file.
# shellcheck disable=SC2034
esc=$'\e'
scenario_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
run_dir=${MEIGA_PERF_RUN:-$scenario_dir/../../../.perf/run}
# Written when the scenario ends, before the terminal is reset.
scenario_end_text=
# Files to delete when the scenario ends.
scenario_temp_files=()

# Sets now_us to the microseconds since the scenario started. EPOCHREALTIME has
# 6 decimals, and its decimal separator depends on the locale.
scenario_now() {
  local now=${EPOCHREALTIME/[.,]/}
  now_us=$((now - start_us))
}

# Prepares the terminal and, under run-scenario.ps1, says that the scenario
# has started. $1 is the name of the scenario; with "keep-cursor" as $2, the
# cursor stays visible, as with a program that only prints text.
scenario_start() {
  scenario_name=$1
  scenario_keep_cursor=${2:-}
  start_us=${EPOCHREALTIME/[.,]/}
  now_us=0
  next_frame_us=0
  next_stop_check_us=0
  scenario_marker=
  scenario_controlled=false
  if [[ -f $run_dir/controller ]]; then
    scenario_controlled=true
  fi
  # A pipe that never gets data: a read with a timeout on it waits without
  # starting a process, which sleep would do in every frame.
  exec {sleep_fd}<> <(:)
  trap scenario_end EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  if [[ $scenario_keep_cursor != keep-cursor ]]; then
    # Hide the cursor while drawing, as command-line tools do.
    printf '\e[?25l'
  fi
  if $scenario_controlled; then
    scenario_marker=$run_dir/started-$scenario_name-bash-$$
    : >"$scenario_marker"
  fi
}

# Leaves the terminal as it was and deletes the start marker. It runs on exit,
# also after Ctrl+C.
scenario_end() {
  printf '%s\e[0m' "$scenario_end_text"
  if [[ $scenario_keep_cursor != keep-cursor ]]; then
    printf '\e[?25h'
  fi
  if [[ -n $scenario_marker ]]; then
    scenario_temp_files+=("$scenario_marker")
  fi
  if ((${#scenario_temp_files[@]} > 0)); then
    rm -f -- "${scenario_temp_files[@]}"
  fi
}

# Succeeds when run-scenario.ps1 has asked the scenario to stop. It looks for
# the file at most once per second: in WSL, every look crosses to Windows.
scenario_should_stop() {
  $scenario_controlled || return 1
  scenario_now
  ((now_us >= next_stop_check_us)) || return 1
  next_stop_check_us=$((now_us + 1000000))
  [[ -f $run_dir/stop ]]
}

# Sleeps $1 seconds, a number that read -t accepts, such as 0.1.
scenario_sleep() {
  read -r -t "$1" -u "$sleep_fd" _ || true
}

# Sleeps until the next frame, $1 microseconds after the previous one. Frames
# are due at fixed moments since the start of the scenario, so the rate stays
# exact although every sleep lasts a little longer than asked. A frame that
# comes very late moves the next moments forward instead of causing a burst of
# frames.
scenario_wait_frame() {
  local wait_us seconds
  scenario_now
  next_frame_us=$((next_frame_us + $1))
  if ((next_frame_us < now_us)); then
    next_frame_us=$now_us
  fi
  wait_us=$((next_frame_us - now_us))
  ((wait_us > 0)) || return 0
  printf -v seconds '%d.%06d' $((wait_us / 1000000)) $((wait_us % 1000000))
  scenario_sleep "$seconds"
}
