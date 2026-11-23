#!/bin/sh
# A story drawn without a compositor (no window opens) and written to a PNG.
#   tools/shot.sh --list
#   tools/shot.sh <Group/Story> <out.png> [--size WxH] [--scale S] \
#       [--set "Control=value"]... [--select TEXT] [key=value]...
# key=value are the keys of the [appearance] table (mode, density, shape,
# material, motion, theme, ...); mode is dark unless given. e.g.
#   tools/shot.sh "Buttons/Button" /tmp/button.png mode=light density=compact
set -e
cd "$(dirname "$0")/.."
exec cargo run --release --quiet -p styletto-storybook -- shot "$@"
