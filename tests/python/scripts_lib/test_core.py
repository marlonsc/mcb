"""Tests for mcb_scripts.core — FLEXT-style kernel.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, cast

import pytest
from pydantic import BaseModel

from mcb_scripts.core import McbResult, McbService, configure_logging, get_logger, r, s
from tests.python.scripts_lib._utilities.matchers import tm

if TYPE_CHECKING:
    from flext_core import p
    from tests.python.scripts_lib.conftest import SettingsFactory

_SEED = 21
_OK = 42
_THRESHOLD = 10
_BOOM_LENGTH = 4
_RECOVERED = 7
_OR_ELSE = 4


class TestResultBasics:
    """Define ``TestResultBasics``."""
    @staticmethod
    def test_ok_is_ok() -> None:
        """Test ok is ok."""
        result: p.Result[int] = r[int].ok(_OK)
        tm.ok(result, _OK)
        assert result.value == _OK
        assert bool(result)
        assert repr(result) == "r[T].ok(42)"

    @staticmethod
    def test_err_is_err() -> None:
        """Test err is err."""
        result: p.Result[int] = r[int].fail("boom", error_code="E001")
        tm.fail(result)
        assert not bool(result)
        assert result.error == "boom"
        assert result.error_code == "E001"
        assert repr(result) == "r[T].fail('boom')"

    @staticmethod
    def test_unwrap_raises_runtime_error() -> None:
        """Test unwrap raises runtime error."""
        result: p.Result[int] = r[int].fail("boom")
        with pytest.raises(RuntimeError, match="boom"):
            result.unwrap()

    @staticmethod
    def test_or_operator() -> None:
        """Test or operator."""
        assert (r[int].ok(_OK) | 0) == _OK
        assert (r[int].fail("boom") | 0) == 0

    @staticmethod
    def test_context_manager() -> None:
        """Test context manager."""
        with r[int].ok(_OK) as value:
            tm.ok(value, _OK)

    @staticmethod
    def test_unwrap_or() -> None:
        """Test unwrap or."""
        fallback = 0
        assert r[int].ok(_OK).unwrap_or(fallback) == _OK
        assert r[int].fail("boom").unwrap_or(fallback) == fallback

    @staticmethod
    def test_unwrap_or_else() -> None:
        """Test unwrap or else."""
        assert r[int].ok(_OK).unwrap_or_else(lambda: 0) == _OK
        assert r[int].fail("boom").unwrap_or_else(lambda: _OR_ELSE) == _OR_ELSE


class TestResultCombinators:
    """Define ``TestResultCombinators``."""
    @staticmethod
    def test_map() -> None:
        """Test map."""
        assert r[int].ok(_SEED).map(lambda x: x * 2).unwrap() == _OK
        mapped = r[int].fail("boom").map(lambda x: x * 2)
        tm.fail(mapped)

    @staticmethod
    def test_flat_map() -> None:
        """Test flat map."""
        def double(x: int) -> p.Result[int]:
            return r[int].ok(x * 2)

        assert r[int].ok(_SEED).flat_map(double).unwrap() == _OK
        chained = r[int].ok(_SEED).flat_map(double).flat_map(double)
        tm.ok(chained, 84)

        failed = r[int].fail("boom").flat_map(double)
        tm.fail(failed)

    @staticmethod
    def test_map_catches_exceptions_as_failure() -> None:
        """Test map catches exceptions as failure."""
        def _boom(_: int) -> int:
            msg = "map must catch"
            raise ZeroDivisionError(msg)

        result = r[int].ok(_SEED).map(_boom)
        tm.fail(result)
        assert result.error == "map must catch"

    @staticmethod
    def test_fold() -> None:
        """Test fold."""
        ok_result = r[int].ok(_SEED)
        assert ok_result.fold(lambda _: -1, lambda v: v * 2) == _OK

        err_result = r[int].fail("boom")
        assert err_result.fold(len, lambda _: 0) == _BOOM_LENGTH

    @staticmethod
    def test_recover() -> None:
        """Test recover."""
        assert r[int].ok(_OK).recover(lambda _: 0).unwrap() == _OK
        assert r[int].fail("boom").recover(lambda _: _RECOVERED).unwrap() == _RECOVERED

    @staticmethod
    def test_lash() -> None:
        """Test lash."""
        assert r[int].ok(_OK).lash(lambda _: r[int].ok(99)).unwrap() == _OK
        recovered = r[int].fail("boom").lash(lambda _: r[int].ok(_RECOVERED))
        tm.ok(recovered, _RECOVERED)

    @staticmethod
    def test_filter() -> None:
        """Test filter."""
        assert r[int].ok(_OK).filter(lambda x: x > _THRESHOLD).unwrap() == _OK
        filtered = r[int].ok(5).filter(lambda x: x > _THRESHOLD)
        tm.fail(filtered)
        assert r[int].fail("boom").filter(lambda x: x > _THRESHOLD).failure


class TestResultAdvanced:
    """Define ``TestResultAdvanced``."""
    @staticmethod
    def test_flow_through() -> None:
        """Test flow through."""
        def add_one(x: int) -> p.Result[int]:
            return r[int].ok(x + 1)

        result = r[int].ok(1).flow_through(add_one, add_one, add_one)
        tm.ok(result, 4)

        def fail(_x: int) -> p.Result[int]:
            return r[int].fail("stop")

        halted = r[int].ok(1).flow_through(add_one, fail, add_one)
        tm.fail(halted)

    @staticmethod
    def test_tap() -> None:
        """Test tap."""
        side_effect: list[int] = []
        result = r[int].ok(_OK).tap(side_effect.append)
        tm.ok(result, _OK)
        assert side_effect == [_OK]

        r[int].fail("boom").tap(side_effect.append)
        assert side_effect == [_OK]

    @staticmethod
    def test_tap_error() -> None:
        """Test tap error."""
        side_effect: list[str] = []
        result = r[int].fail("boom").tap_error(side_effect.append)
        tm.fail(result)
        assert side_effect == ["boom"]

        r[int].ok(_OK).tap_error(side_effect.append)
        assert side_effect == ["boom"]

    @staticmethod
    def test_map_error() -> None:
        """Test map error."""
        result = r[int].fail("boom").map_error(lambda e: e.upper())
        assert result.error == "BOOM"

        unchanged = r[int].ok(_OK).map_error(lambda e: e.upper())
        tm.ok(unchanged, _OK)

    @staticmethod
    def test_map_or() -> None:
        """Test map or."""
        assert r[int].ok(_OK).map_or(0) == _OK
        assert r[int].fail("boom").map_or(0) == 0

        def _double(value: int) -> int:
            return value * 2

        assert r[int].ok(_SEED).map_or(0, _double) == _OK
        assert r[int].fail("boom").map_or(0, _double) == 0

    @staticmethod
    def test_fail_op() -> None:
        """Test fail op."""
        result = r[int].fail_op("load")
        tm.fail(result, "load failed")

        with_exception = r[int].fail_op("load", ValueError("missing"))
        tm.fail(with_exception, "load failed: missing")
        assert isinstance(with_exception.exception, ValueError)

    @staticmethod
    def test_from_validation() -> None:
        """Test from validation."""
        class User(BaseModel):
            name: str
            age: int

        result = McbResult.from_validation({"name": "Ada", "age": 36}, User)
        tm.ok(result)
        assert result.value.name == "Ada"

        invalid = McbResult.from_validation({"name": "Ada"}, User)
        tm.fail(invalid)

    @staticmethod
    def test_accumulate_errors() -> None:
        """Test accumulate errors."""
        results = [r[int].ok(1), r[int].ok(2), r[int].ok(3)]
        combined = McbResult.accumulate_errors(*results)
        tm.ok(combined)
        assert list(combined.value) == [1, 2, 3]

        mixed = [r[int].ok(1), r[int].fail("a"), r[int].fail("b")]
        combined = McbResult.accumulate_errors(*mixed)
        tm.fail(combined, "a")
        tm.fail(combined, "b")

    @staticmethod
    def test_safe_decorator() -> None:
        """Test safe decorator."""
        @McbResult.safe
        def double(x: int) -> int:
            return x * 2

        assert double(_SEED).unwrap() == _OK

        @McbResult.safe
        def explode() -> int:
            msg = "boom"
            raise ValueError(msg)

        result = explode()
        tm.fail(result, "boom")


class TestSettings:
    """Define ``TestSettings``."""
    @staticmethod
    def test_base_settings_read_env_with_prefix(
        monkeypatch: pytest.MonkeyPatch, settings_factory: SettingsFactory,
    ) -> None:
        """Test base settings read env with prefix."""
        monkeypatch.setenv("MCB_LOG_LEVEL", "debug")
        settings = settings_factory(log_level="info")
        assert settings.log_level == "debug"

    @staticmethod
    def test_base_settings_ignore_extra_env(
        monkeypatch: pytest.MonkeyPatch, settings_factory: SettingsFactory,
    ) -> None:
        """Test base settings ignore extra env."""
        monkeypatch.setenv("MCB_UNKNOWN_VAR", "ignored")
        settings = settings_factory(log_level="info")
        assert settings.log_level == "info"

    @staticmethod
    def test_singleton_fetch_global(settings_factory: SettingsFactory) -> None:
        """Test singleton fetch global."""
        settings = settings_factory(name="default")
        first = settings.fetch_global()
        second = settings.fetch_global()
        assert first is second

    @staticmethod
    def test_clone_is_isolated(settings_factory: SettingsFactory) -> None:
        """Test clone is isolated."""
        settings = settings_factory(name="default")
        global_settings = settings.fetch_global()
        clone = global_settings.clone(name="cloned")
        assert clone.model_dump()["name"] == "cloned"
        assert global_settings.model_dump()["name"] == "default"

    @staticmethod
    def test_update_global_propagates(settings_factory: SettingsFactory) -> None:
        """Test update global propagates."""
        settings = settings_factory(name="default")
        settings.update_global(name="updated")
        assert settings.fetch_global().model_dump()["name"] == "updated"

    @staticmethod
    def test_validate_overrides_rejects_unknown(
        settings_factory: SettingsFactory,
    ) -> None:
        """Test validate overrides rejects unknown."""
        settings = settings_factory(name="default")
        with pytest.raises(ValueError, match="Unknown settings override"):
            settings.clone(unknown="value")


class TestLogging:
    """Define ``TestLogging``."""
    @staticmethod
    def test_configure_logging_runs() -> None:
        """Test configure logging runs."""
        configure_logging(json_format=False)
        logger = get_logger(__name__)
        log_result = logger.info("test.event", key="value")
        assert log_result.success

    @staticmethod
    def test_logger_bind() -> None:
        """Test logger bind."""
        configure_logging(json_format=False)
        logger = get_logger(__name__).bind(request_id="abc")
        assert logger.info("bound.event").success


class TestService:
    """Define ``TestService``."""
    @staticmethod
    def test_service_singleton() -> None:
        """Test service singleton."""
        class DemoService(McbService):
            pass

        DemoService.reset_for_testing()
        first = DemoService.fetch_global()
        second = DemoService.fetch_global()
        assert first is second
        DemoService.reset_for_testing()

    @staticmethod
    def test_service_execute_not_implemented() -> None:
        """Test service execute not implemented."""
        class DemoService(McbService):
            pass

        DemoService.reset_for_testing()
        service = DemoService.fetch_global()
        with pytest.raises(NotImplementedError):
            service.execute()
        DemoService.reset_for_testing()

    @staticmethod
    def test_service_alias() -> None:
        """Test service alias."""
        class SettingsService(s):
            pass

        SettingsService.reset_for_testing()
        assert isinstance(SettingsService.fetch_global(), McbService)
        SettingsService.reset_for_testing()

    @staticmethod
    def test_service_with_settings(settings_factory: SettingsFactory) -> None:
        """Test service with settings."""
        settings = settings_factory(name="default")

        class DemoService(McbService):
            pass

        DemoService.reset_for_testing()
        service = DemoService.with_settings(settings.fetch_global())
        runtime_settings = cast("Any", service.runtime_settings)
        assert runtime_settings.name == "default"
        DemoService.reset_for_testing()
