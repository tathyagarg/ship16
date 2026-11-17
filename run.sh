#!/usr/bin/env bash
set -Eeuo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <program-name>" >&2
  exit 2
fi

name="$1"

# Restrict names to safe filename characters.
if [[ ! "$name" =~ ^[a-zA-Z0-9_-]+$ ]]; then
  echo "Error: Invalid program name: $name" >&2
  exit 2
fi

src="roms/asm/${name}.s"
rom="roms/${name}.rom"

if [[ ! -f "$src" ]]; then
  echo "Error: Source file not found: $src" >&2
  exit 1
fi

cargo run -p asm -- \
  --input "$src" \
  --output "$rom"

SHIP16_LOG="emulator.log" cargo run -p ship16-rs -- \
  --rom "$rom" \
  --debug
