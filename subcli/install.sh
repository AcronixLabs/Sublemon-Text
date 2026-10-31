#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
prefix=${PREFIX:-"$HOME/.local"}
share_dir=$prefix/share/subcli
bin_dir=$prefix/bin

mkdir -p "$share_dir" "$bin_dir"
cp -R "$script_dir/init.lua" "$script_dir/lua" "$script_dir/README.md" "$share_dir/"
cp "$script_dir/bin/subcli" "$bin_dir/subcli"
chmod 755 "$bin_dir/subcli"

printf 'Installed subcli to %s\n' "$share_dir"
printf 'Run: %s\n' "$bin_dir/subcli"
