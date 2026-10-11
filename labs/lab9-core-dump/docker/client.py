#!/usr/bin/env python3
"""Tiny client for lab9-core-dump's length-prefixed frame protocol.

Wire format: a 4-byte big-endian length prefix, then that many bytes of
UTF-8 text. Request bodies look like "METRIC <name> <value>"; responses
are "OK" or "ERR <reason>".

Usage:
    python3 client.py <port> "<frame text>" ["<frame text>" ...]

Example:
    python3 client.py 9000 "METRIC cpu.load 0.42" "GARBAGE"

Every frame is sent on the same connection, in order; the response to
each (or a timeout / EOF note) is printed before the next is sent. No
response within `--timeout` seconds (default 3) is reported as a
TIMEOUT rather than raising, since a hung response *is* the interesting
observation for the deadlock fault.
"""

import argparse
import socket
import struct
import sys


def send_frame(sock: socket.socket, text: str) -> None:
    body = text.encode()
    sock.sendall(struct.pack(">I", len(body)) + body)


def recv_exact(sock: socket.socket, n: int) -> bytes | None:
    buf = b""
    while len(buf) < n:
        chunk = sock.recv(n - len(buf))
        if not chunk:
            return None
        buf += chunk
    return buf


def recv_frame(sock: socket.socket) -> bytes | None:
    len_buf = recv_exact(sock, 4)
    if len_buf is None:
        return None
    (length,) = struct.unpack(">I", len_buf)
    return recv_exact(sock, length)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("port", type=int)
    parser.add_argument("frames", nargs="+")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--timeout", type=float, default=3.0)
    args = parser.parse_args()

    sock = socket.create_connection((args.host, args.port), timeout=args.timeout)
    for frame in args.frames:
        print(f"--> {frame!r}")
        send_frame(sock, frame)
        sock.settimeout(args.timeout)
        try:
            response = recv_frame(sock)
        except socket.timeout:
            print(f"<-- TIMEOUT (no response within {args.timeout}s)")
            continue
        if response is None:
            print("<-- EOF (connection closed)")
            break
        print(f"<-- {response!r}")
    sock.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
