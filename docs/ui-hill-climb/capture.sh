#!/bin/bash
# Capture every screen of the demo in light and dark mode.
#
#   docs/ui-hill-climb/capture.sh round-0 [output-root]
#
# Screenshots go to <output-root>/<round>/{light,dark}/, by default
# docs/ui-hill-climb/. Rounds that are not committed can go elsewhere,
# e.g. /tmp/wren-hill-climb.
#
# Needs a release build (cargo build --locked --release). On a headless
# Linux machine it runs each screenshot under its own Xvfb (xvfb-run).
set -euo pipefail
cd "$(dirname "$0")/../.."
round="${1:?usage: capture.sh round-N}"
binary="${WREN:-target/release/wren}"
out="${2:-docs/ui-hill-climb}/$round"

# name | --page parts (mode is appended) | size
shots=(
  "chats|chats|1280x800"
  "chats-wide|chats|1920x1080"
  "chats-narrow|chats|760x640"
  "chat-no-selection|no-chat|1280x800"
  "chat-filter-no-match|no-match|1280x800"
  "group-chat|group|1280x800"
  "channel|channel|1280x800"
  "channel-narrow|channel|760x640"
  "teams|teams|1280x800"
  "calendar|calendar|1280x800"
  "calls|calls|1280x800"
  "calls-narrow|calls|760x640"
  "composer-mention|mention|1280x800"
  "composer-long-draft|long-draft|1280x800"
  "composer-images|images|1280x800"
  "image-preview|image-preview|1280x800"
  "search-switcher|switcher|1280x800"
  "new-chat|newchat|1280x800"
  "settings|settings|1280x800"
  "shortcuts|shortcuts|1280x800"
  "status-message|status|1280x800"
  "account-menu|accounts|1280x800"
  "permission-consent|consent|1280x800"
  "error-toast|error|1280x800"
  "first-sync-loading|loading|1280x800"
  "first-sync-empty|empty|1280x800"
  "call-bar|callbar|1280x800"
  "call-docked|call|720x800"
  "join-on-phone|phone|1280x800"
  "setup|setup|1280x800"
  "signin|signin|1280x800"
  "signin-browser|signin-browser|1280x800"
  "signin-device-code|signin-code|1280x800"
  "signin-finishing|signin-wait|1280x800"
  "signin-expired|signin-expired|1280x800"
  "signin-failed|signin-failed|1280x800"
)

run() {
  local mode="$1" name="$2" page="$3" size="$4"
  local width="${size%x*}" height="${size#*x}"
  mkdir -p "$out/$mode"
  # The demo builds its sample data around the current time. Software GL
  # under Xvfb takes a few seconds to the first frame, and dialogs fade in
  # after that, so wait well past both before the screenshot.
  xvfb-run -a -s "-screen 0 $((width + 64))x$((height + 64))x24" \
    env -u WAYLAND_DISPLAY LANG=en_US.UTF-8 \
    "$binary" --screenshot "$out/$mode/$name.png" --page "$page,$mode" --size "$size" \
    --screenshot-delay "${DELAY:-9000}" \
    >/dev/null 2>&1 || echo "failed: $mode/$name" >&2
}

jobs=0
for mode in light dark; do
  for shot in "${shots[@]}"; do
    IFS='|' read -r name page size <<<"$shot"
    run "$mode" "$name" "$page" "$size" &
    jobs=$((jobs + 1))
    if [ "$jobs" -ge 6 ]; then
      wait -n
      jobs=$((jobs - 1))
    fi
  done
done
wait
ls "$out"/light "$out"/dark | grep -c png
