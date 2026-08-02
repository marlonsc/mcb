"""Tests for external service availability checker.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
from __future__ import annotations

import socket
from pathlib import Path
from unittest import mock

from lib.external_services_check import is_reachable, load_services, main, parse_url

ROOT = Path(__file__).resolve().parents[3]


def test_parse_url_extracts_host_and_port() -> None:
    assert parse_url("http://localhost:19530") == ("localhost", 19530)
    assert parse_url("redis://localhost:26379") == ("localhost", 26379)
    assert parse_url("postgresql://user:pass@localhost:25432/db") == ("localhost", 25432)


def test_parse_url_returns_none_when_port_missing() -> None:
    assert parse_url("http://localhost") is None


def test_is_reachable_false_when_connection_refused() -> None:
    # Port 0 is not a valid target; should fail immediately.
    assert is_reachable("localhost", 0, timeout=0.1) is False


def test_is_reachable_true_on_open_port(tmp_path: Path) -> None:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.bind(("127.0.0.1", 0))
    sock.listen(1)
    _, port = sock.getsockname()
    try:
        assert is_reachable("127.0.0.1", port, timeout=0.5) is True
    finally:
        sock.close()


def test_load_services_returns_dict_from_config() -> None:
    services = load_services()
    assert isinstance(services, dict)
    # config/tests.toml contains commented-out keys by default, so the dict
    # is empty in a fresh checkout.
    assert all(isinstance(k, str) and isinstance(v, str) for k, v in services.items())


def test_main_returns_one_when_no_services_configured(tmp_path: Path) -> None:
    config = tmp_path / "tests.toml"
    config.write_text("[test_services]\n", encoding="utf-8")
    with mock.patch("lib.external_services_check.CONFIG_PATH", config):
        assert main() == 1


def test_main_returns_one_when_service_unreachable(tmp_path: Path) -> None:
    config = tmp_path / "tests.toml"
    config.write_text(
        "[test_services]\nmilvus = \"http://localhost:1\"\n",
        encoding="utf-8",
    )
    with mock.patch("lib.external_services_check.CONFIG_PATH", config):
        assert main() == 1
