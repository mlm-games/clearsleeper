#!/usr/bin/env bash
# Enforces the layering: the engine layers read data, the game reads the engine.
#
# src/archive and src/j3d hold the GameCube container and scene formats. They are the only layers
# below the game, so they must not name a game type. A violation compiles fine and fails at
# runtime, which is what the .bdl4 extension in a Wind Waker loader did, so this is checked rather
# than left to review.
#
# doc comments name the forbidden crates when they explain why a thing sits here, so the pattern
# skips lines whose first non-space character is `//`, `///`, `//!` or `*`.

set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

layers=(src/archive.rs src/j3d)
forbidden='crate::(sim|ui|ui_[a-z_]+|assets|save|view|lib)::'
status=0

for layer in "${layers[@]}"; do
    if [ ! -e "$layer" ]; then
        echo "layering: missing layer $layer"
        status=1
        continue
    fi
    hits=$(grep -rnE "$forbidden" "$layer" --include='*.rs' \
        | grep -vE '^[^:]+:[0-9]+:\s*(//|///|//!|\*)' \
        || true)
    if [ -n "$hits" ]; then
        echo "layering: $layer names a game type:"
        echo "$hits"
        status=1
    fi
done

if [ "$status" -eq 0 ]; then
    echo "layering: ok (${layers[*]})"
fi

exit "$status"
