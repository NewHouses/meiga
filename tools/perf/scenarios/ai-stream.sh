#!/usr/bin/env bash
# Load scenario: imitates an AI assistant that answers in a terminal. The same
# as ai-stream.ps1, for bash, and it writes the same bytes: see that file for
# the details.
#
# Usage: bash ai-stream.sh [words per second [seconds of thinking]]
set -uo pipefail
# shellcheck source=common.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/common.sh"

words_per_second=${1:-40}
think_us=$((${2:-2} * 1000000))

# The conversation: prompts[n] is prompt n, and its answer is tokens[first[n]]
# to tokens[first[n + 1] - 1]. A token is a word with what goes before it,
# that is, the spaces before it and, at the start of a line, the line breaks.
# Blank lines at the start and at the end of an answer are left out, because
# the scenario draws its own.
prompts=()
first=()
tokens=()
read_conversation() {
  local line rest prefix i breaks=0 n=-1 word_re='^( *[^ ]+)(.*)$'
  while IFS= read -r line || [[ -n $line ]]; do
    line=${line%$'\r'}
    line=${line//\\e/$esc}
    if [[ $line == '> '* ]]; then
      n=$((n + 1))
      prompts[n]=${line:2}
      first[n]=${#tokens[@]}
      breaks=0
      continue
    fi
    ((n >= 0)) || continue
    if [[ -z ${line// /} ]]; then
      breaks=$((breaks + 1))
      continue
    fi
    prefix=
    if ((${#tokens[@]} > first[n])); then
      for ((i = 0; i <= breaks; i++)); do
        prefix+=$'\n'
      done
    fi
    breaks=0
    rest=$line
    while [[ $rest =~ $word_re ]]; do
      tokens+=("$prefix${BASH_REMATCH[1]}")
      prefix=
      rest=${BASH_REMATCH[2]}
    done
  done <"$scenario_dir/ai-stream.txt"
  if ((n < 0)); then
    echo "ai-stream.txt has no prompts, that is, lines that start with '> '." >&2
    exit 1
  fi
  first[n + 1]=${#tokens[@]}
}

read_conversation
count=${#prompts[@]}
glyphs=(⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏)
rule=
for ((i = 0; i < 60; i++)); do
  rule+=─
done
hint="${esc}[2mctrl+c to stop · simulated AI answer${esc}[0m"

# prompt: show the next prompt; thinking: before the answer; streaming: while
# writing it; answered: the answer has finished.
phase=prompt
exchange=0
exchange_start_us=0
emitted=0
# The last line of text, which every frame writes again.
partial=

scenario_start ai-stream
frame=0
while ! scenario_should_stop; do
  scenario_now
  out=
  if ((frame > 0)); then
    # Back to the start of the last line of text, 3 lines above the end of
    # the status area, and erase down to the end of the screen.
    out=$'\r'"${esc}[3A${esc}[J"
  fi
  out+=$partial
  if [[ $phase == answered ]]; then
    exchange=$(((exchange + 1) % count))
    phase=prompt
  fi
  total=$((first[exchange + 1] - first[exchange]))
  if [[ $phase == prompt ]]; then
    out+="${esc}[1;36m❯${esc}[0m ${esc}[1m${prompts[exchange]}${esc}[0m"$'\n\n'
    phase=thinking
    exchange_start_us=$now_us
    emitted=0
  elif ((now_us >= exchange_start_us + think_us)); then
    phase=streaming
    due=$(((now_us - exchange_start_us - think_us) * words_per_second / 1000000))
    if ((due > total)); then
      due=$total
    fi
    while ((emitted < due)); do
      token=${tokens[first[exchange] + emitted]}
      out+=$token
      if [[ $token == *$'\n'* ]]; then
        partial=${token##*$'\n'}
      else
        partial+=$token
      fi
      emitted=$((emitted + 1))
    done
    if ((emitted == total)); then
      # End the last line and leave a blank line before the next prompt.
      out+=$'\n\n'
      partial=
      phase=answered
    fi
  fi
  seconds=$(((now_us - exchange_start_us) / 1000000))
  if [[ $phase == thinking ]]; then
    state="Thinking…${esc}[0m ${esc}[2m($seconds s)"
  else
    state="Generating…${esc}[0m ${esc}[2m($seconds s · $emitted words)"
  fi
  spinner="${esc}[38;5;$((16 + frame % 216))m${glyphs[frame % 10]}${esc}[0m"
  out+="${esc}[0m"$'\n'"$spinner ${esc}[1m$state${esc}[0m"$'\n'"${esc}[2m$rule${esc}[0m"$'\n'"$hint"
  printf '%s' "$out"
  # Erase the status area and leave the cursor below the text.
  scenario_end_text=$'\r'"${esc}[3A${esc}[J$partial${esc}[0m"$'\n'
  frame=$((frame + 1))
  scenario_wait_frame 100000
done
