"""Test constants for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
tests/constants
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_tests import FlextTestsConstants

from mcb_scripts import c


class TestsMcbScriptsConstants(FlextTestsConstants, c):
    """Test constants for mcbscripts."""

    class TestsMcbScripts(FlextTestsConstants.Tests, c.McbScripts):
        """Test namespace composed from shared and project contracts."""


c = TestsMcbScriptsConstants

__all__: list[str] = ["TestsMcbScriptsConstants", "c"]
