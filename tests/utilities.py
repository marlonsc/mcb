"""Test utilities for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
tests/utilities
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_tests import FlextTestsUtilities

from mcb_scripts import u


class TestsMcbScriptsUtilities(FlextTestsUtilities, u):
    """Test utilities for mcbscripts."""

    class TestsMcbScripts(FlextTestsUtilities.Tests, u.McbScripts):
        """Test namespace composed from shared and project contracts."""


u = TestsMcbScriptsUtilities

__all__: list[str] = ["TestsMcbScriptsUtilities", "u"]
