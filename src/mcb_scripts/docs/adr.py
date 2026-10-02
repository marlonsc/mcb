"""Render architecture decision records from the repository-owned template.

Copyright (c) 2026 Marlon Costa. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import os
import re
import tempfile
from pathlib import Path

_ADR_NAME = re.compile(r"^(?P<number>[0-9]{3})-[a-z0-9-]+\.md$")
_STATUS = re.compile(
    r"^(?:Proposed|Accepted|Rejected|Deprecated|Superseded by ADR-[0-9]{3})$",
)
_SLOTS = (
    "{number}",
    "{title}",
    "{Proposed | Accepted | Rejected | Deprecated | Superseded by ADR-xxx}",
)


def create_adr(root: Path, title: str, status: str, *, dry_run: bool = False) -> Path:
    """Render and publish one ADR without replacing an existing record.

    Returns:
        The resulting ``Path``.

    Raises:
        FileExistsError: If ``destination.exists()``.
        ValueError: If ADR title must be nonempty and contain one line; or if Invalid
            ADR status; or if ADR template must contain each declared slot once; or if
            ADR title must contain an ASCII letter or digit.
    """
    if not title.strip() or "\n" in title or "\r" in title:
        msg = "ADR title must be nonempty and contain one line"
        raise ValueError(msg)
    if not _STATUS.fullmatch(status):
        msg = f"Invalid ADR status: {status!r}"
        raise ValueError(msg)

    directory = root / "docs" / "adr"
    template = root / "docs" / "templates" / "adr-template.md"
    source = template.read_text(encoding="utf-8")
    if any(source.count(slot) != 1 for slot in _SLOTS):
        msg = f"ADR template must contain each declared slot once: {template}"
        raise ValueError(msg)

    numbers = (
        int(match.group("number"))
        for path in directory.iterdir()
        if (match := _ADR_NAME.fullmatch(path.name)) is not None
    )
    number = f"{max(numbers, default=0) + 1:03d}"
    slug = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    if not slug:
        msg = "ADR title must contain an ASCII letter or digit"
        raise ValueError(msg)
    destination = directory / f"{number}-{slug}.md"
    if destination.exists():
        raise FileExistsError(destination)

    rendered = source
    for slot, value in zip(_SLOTS, (number, title, status), strict=True):
        rendered = rendered.replace(slot, value)
    if dry_run:
        return destination

    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=directory, prefix=".adr-stage-", delete=True,
    ) as staged:
        staged.write(rendered)
        staged.flush()
        os.fsync(staged.fileno())
        Path(staged.name).chmod(0o644)
        os.link(staged.name, destination)
    return destination
