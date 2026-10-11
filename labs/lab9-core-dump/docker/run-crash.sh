#!/usr/bin/env bash
# Capture a live Linux deadlock as a core snapshot. The historical filename
# is retained; a hang does not itself crash, and this uses GDB rather than
# changing kernel-global core_pattern or sending SIGABRT.
set -euo pipefail

if [ "$(uname -s)" != Linux ]; then
    echo "Run this script inside the lab's Linux Docker container." >&2
    exit 1
fi
WORKSPACE_ROOT="${WORKSPACE_ROOT:-/workspace}"
TARGET_DIR="${CARGO_TARGET_DIR:-$WORKSPACE_ROOT/target}"
BIN="$TARGET_DIR/release/lab9-core-dump-starter"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="${RUN_DIR:-/tmp/lab9-crash}"
PORT="${METRICS_PORT:-9000}"
HANG_TIMEOUT="${HANG_TIMEOUT:-5}"
mkdir -p "$RUN_DIR/cores"
RUN_DIR="$(cd "$RUN_DIR" && pwd)"
SERVER_LOG="$RUN_DIR/server.log"
CORE_FILE="$RUN_DIR/cores/core.deadlock"
command -v gdb >/dev/null
if [ ! -x "$BIN" ]; then
    echo "Missing release binary: $BIN" >&2
    exit 1
fi
# Refuse stale artifacts instead of mistaking an earlier dump for success.
if [ -e "$CORE_FILE" ]; then
    echo "Core already exists: $CORE_FILE; use a fresh RUN_DIR." >&2
    exit 1
fi
SERVER_PID=""
cleanup() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT
FAULT=deadlock RUST_LOG=info METRICS_PORT="$PORT" "$BIN" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!
# Poll readiness, require a real timeout, and verify the process is still live.
python3 - "$PORT" "$HANG_TIMEOUT" "$SCRIPT_DIR/client.py" "$SERVER_LOG" <<'PYCLIENT'
import importlib.util, socket, sys, time
from pathlib import Path
port, timeout, path = int(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
spec = importlib.util.spec_from_file_location('lab_client', path)
client = importlib.util.module_from_spec(spec)
spec.loader.exec_module(client)
deadline = time.monotonic() + 10
log = Path(sys.argv[4])
while True:
    evidence = log.read_text()
    if 'task_a: attempting registry lock' in evidence and 'task_b: attempting metrics lock' in evidence:
        break
    if time.monotonic() >= deadline:
        raise SystemExit('Deadlock setup not observed; inspect server.log.')
    time.sleep(0.1)
while True:
    try:
        sock = socket.create_connection(('127.0.0.1', port), timeout=timeout)
        break
    except OSError:
        if time.monotonic() >= deadline:
            raise SystemExit('Server did not listen within 10 seconds; inspect server.log.')
        time.sleep(0.1)
with sock:
    client.send_frame(sock, 'METRIC cpu.load 0.42')
    try:
        response = client.recv_frame(sock)
    except socket.timeout:
        print('TIMEOUT observed; capture the snapshot and correlate with logs.')
    else:
        raise SystemExit(f'Expected a hung request, got {response!r}; no core captured.')
PYCLIENT
kill -0 "$SERVER_PID"
# Attach requires SYS_PTRACE and a compatible seccomp policy in Docker.
# generate-core-file snapshots a live process; ulimit/core_pattern are unused.
gdb -q -nx -batch "$BIN" \
    -ex "attach $SERVER_PID" \
    -ex "generate-core-file $CORE_FILE" \
    -ex detach -ex quit > "$RUN_DIR/capture.log" 2>&1 || {
        cat "$RUN_DIR/capture.log" >&2
        exit 1
    }
if [ ! -s "$CORE_FILE" ]; then
    cat "$RUN_DIR/capture.log" >&2
    echo "No core snapshot produced." >&2
    exit 1
fi
cleanup
SERVER_PID=""
echo "Core: $CORE_FILE"
echo "Logs: $SERVER_LOG"
echo "Inspect: gdb $BIN $CORE_FILE"
echo "Copy /tmp/lab9-crash and the exact release binary before removing the container."
if [ "${LAB9_INTERACTIVE:-1}" = 1 ]; then
    exec bash
fi
