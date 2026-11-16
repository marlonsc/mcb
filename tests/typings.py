"""Test type aliases for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
tests/typings
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_tests import FlextTestsTypes

from mcb_scripts import t


class TestsMcbScriptsTypes(FlextTestsTypes, t):
    """Test type aliases for mcbscripts."""

    class TestsMcbScripts(FlextTestsTypes.Tests, t.McbScripts):
        """Test namespace composed from shared and project contracts."""


t = TestsMcbScriptsTypes

__all__: list[str] = ["TestsMcbScriptsTypes", "t"]
