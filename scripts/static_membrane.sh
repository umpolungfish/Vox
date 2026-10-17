#!/usr/bin/env bash
# Build-time policy. Emitted membranes do not depend on this helper.
export RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+crt-static"
if [[ -v CARGO_ENCODED_RUSTFLAGS ]]; then
    export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS:+${CARGO_ENCODED_RUSTFLAGS}$'\x1f'}"$'-C\x1ftarget-feature=+crt-static'
fi
static_membrane_checker=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/standalone.py
require_static_membrane() {
    python3 "$static_membrane_checker" "$@"
}
finish_static_membrane() {
    require_static_membrane "$1"
    if [[ ${MEMBRANE_RUN:-0} == 1 ]]; then
        exec "$1"
    fi
    printf '%s\n' "$1"
}
