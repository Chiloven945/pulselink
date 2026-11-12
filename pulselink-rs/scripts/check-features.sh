#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== facade feature checks =="
cargo check -p pulselink --no-default-features
cargo check -p pulselink --no-default-features --features ble
cargo check -p pulselink --no-default-features --features websocket
cargo check -p pulselink --no-default-features --features protocol
cargo check -p pulselink --no-default-features --features full
cargo check -p pulselink --all-features
cargo check -p pulselink --no-default-features --features protocol --example protocol_only

echo "== compile-only checks =="
cargo test --workspace --no-default-features
cargo test -p pulselink-device
cargo test -p pulselink-protocol --no-default-features --features socket-v4
cargo check -p pulselink-protocol --no-default-features --features coyote-v2,opossum,civet,paw-prints

assert_absent() {
    local name="$1"
    local pattern="$2"
    shift 2
    if cargo tree "$@" --edges normal | grep -Eq "$pattern"; then
        echo "error: ${name} unexpectedly contains ${pattern}" >&2
        exit 1
    fi
    echo "ok: ${name} has no ${pattern}"
}

assert_absent "facade/websocket" 'pulselink-(ble|runtime|device)|btleplug' \
    -p pulselink --no-default-features --features websocket
assert_absent "facade/ble" 'pulselink-ws|tokio-tungstenite' \
    -p pulselink --no-default-features --features ble
assert_absent "runtime" 'pulselink-(ble|ws)|btleplug' -p pulselink-runtime
assert_absent "ws" 'pulselink-(ble|runtime|transport)' -p pulselink-ws
assert_absent "protocol" 'tokio|btleplug|tokio-tungstenite' \
    -p pulselink-protocol --no-default-features --features coyote-v3,socket-v4
assert_absent "transport" 'btleplug|tokio-tungstenite' -p pulselink-transport
assert_absent "ble" 'pulselink-(protocol|runtime|ws)' -p pulselink-ble
assert_absent "device" 'pulselink-(ble|ws)|btleplug|tokio-tungstenite' -p pulselink-device

echo "feature matrix ok"
