"""Test models for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
tests/models
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_tests import FlextTestsModels

from mcb_scripts import m


class TestsMcbScriptsModels(FlextTestsModels, m):
    """Test models for mcbscripts."""

    class TestsMcbScripts(FlextTestsModels.Tests, m.McbScripts):
        """Test namespace composed from shared and project contracts."""


m = TestsMcbScriptsModels

__all__: list[str] = ["TestsMcbScriptsModels", "m"]
