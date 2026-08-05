#!/usr/bin/env python3
"""Check external test service availability for dynamic test grouping.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
from __future__ import annotations

import socket
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONFIG_PATH = ROOT / "config" / "tests.toml"


def parse_url(url: str) -> tuple[str, int] | None:
    """Extract host and port from a URL-ish string."""
    rest = url.rsplit("://", 1)[-1]
    rest = rest.rsplit("@", 1)[-1]
    host_port = rest.split("/", 1)[0]
    if ":" not in host_port:
        return None
    host, port_str = host_port.rsplit(":", 1)
    try:
        return host, int(port_str)
    except ValueError:
        return None


def is_reachable(host: str, port: int, timeout: float = 0.3) -> bool:
    """Return True if a TCP connection to host:port succeeds."""
    try:
        with socket.create_connection((host, port), timeout=timeout):
            return True
    except OSError:
        return False


def load_services() -> dict[str, str]:
    """Load configured services from config/tests.toml."""
    if not CONFIG_PATH.is_file():
        return {}
    data = tomllib.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    return dict(data.get("test_services", {}))


def main() -> int:
    """Print availability summary and exit 0 only if all configured services are up."""
    services = load_services()
    if not services:
        print("No external test services configured in config/tests.toml; skipping external test group.")
        return 1

    print("External service availability:")
    all_available = True
    for key, url in sorted(services.items()):
        parsed = parse_url(url)
        available = parsed is not None and is_reachable(*parsed)
        marker = "✓" if available else "✗"
        print(f"  {marker} {key}: {url}")
        if not available:
            all_available = False

    if all_available:
        print("All configured external services are available.")
        return 0

    print("One or more external services are unavailable; skipping external test group.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
