"""Test protocols for mcbscripts.

Copyright (c) 2026 Marlon Costa. All rights reserved.
tests/protocols
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

from flext_tests import FlextTestsProtocols

from mcb_scripts import p


class TestsMcbScriptsProtocols(FlextTestsProtocols, p):
    """Test protocols for mcbscripts."""

    class TestsMcbScripts(FlextTestsProtocols.Tests, p.McbScripts):
        """Test namespace composed from shared and project contracts."""


p = TestsMcbScriptsProtocols

__all__: list[str] = ["TestsMcbScriptsProtocols", "p"]
