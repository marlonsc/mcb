"""Generate the deterministic project metrics page from declared source roots."""

from __future__ import annotations

import os
import re
import tempfile
import tomllib
from pathlib import Path

from jinja2 import Environment, FileSystemLoader, StrictUndefined

_TEST = re.compile(r"#\[\s*(?:tokio::)?test\s*\]")
_ADR = re.compile(r"^[0-9]{3}-[a-z0-9-]+\.md$")


def _declared_dir(root: Path, value: object) -> Path:
    """Resolve only a declared directory inside this repository."""
    if not isinstance(value, str):
        msg = "Docs metrics source path must be a string"
        raise TypeError(msg)
    relative = Path(value)
    if relative.is_absolute() or ".." in relative.parts:
        msg = f"Docs metrics source path escapes the repository: {value!r}"
        raise ValueError(msg)
    directory = root / relative
    if not directory.is_dir():
        raise FileNotFoundError(directory)
    return directory


def _category(root: Path, raw: dict[str, object]) -> tuple[str, ...]:
    """Resolve one configured code category without inventing missing sources."""
    relative = raw["root"]
    excludes = raw["exclude"]
    labels = raw.get("labels", {})
    if not isinstance(excludes, list) or not all(
        isinstance(name, str) for name in excludes
    ):
        msg = "Invalid docs metrics category"
        raise TypeError(msg)
    if not isinstance(labels, dict) or not all(
        isinstance(name, str) and isinstance(label, str)
        for name, label in labels.items()
    ):
        msg = "Invalid docs metrics labels"
        raise TypeError(msg)
    directory = _declared_dir(root, relative)
    names = (
        labels.get(path.stem, path.stem.replace("_", " ").title())
        for path in sorted(directory.glob("*.rs"))
        if path.stem not in excludes
    )
    return tuple(str(name) for name in names)


def render_metrics(root: Path) -> str:
    """Render the complete metrics document from live Rust and docs sources."""
    config_path = root / "config/docs-metrics.toml"
    with config_path.open("rb") as source:
        config = tomllib.load(source)
    categories = config["categories"]
    if not isinstance(categories, dict):
        msg = "Invalid docs metrics categories"
        raise TypeError(msg)
    source_root = _declared_dir(root, config["source_root"])
    adr_dir = _declared_dir(root, config["adr_dir"])
    module_docs_dir = _declared_dir(root, config["module_docs_dir"])
    source_files = tuple(sorted(source_root.rglob("*.rs")))
    if not source_files:
        msg = f"No Rust source files under {source_root}"
        raise ValueError(msg)
    test_files = tuple(
        path for path in source_files if "tests" in path.relative_to(source_root).parts
    )
    test_count = 0
    source_lines = 0
    for path in source_files:
        source = path.read_text(encoding="utf-8")
        test_count += len(_TEST.findall(source))
        source_lines += len(source.splitlines())
    if test_count == 0:
        msg = "No Rust tests found in the declared source root"
        raise ValueError(msg)

    with (root / "Cargo.toml").open("rb") as cargo_source:
        cargo = tomllib.load(cargo_source)
    workspace = cargo["workspace"]
    version = workspace["package"]["version"]
    if not isinstance(version, str) or not version:
        msg = "Workspace version must be a nonempty string"
        raise TypeError(msg)

    environment = Environment(
        loader=FileSystemLoader(root / "docs/templates"),
        undefined=StrictUndefined,
        trim_blocks=True,
        lstrip_blocks=True,
        autoescape=True,
    )
    template = environment.get_template("metrics.md.j2")
    languages = _category(root, categories["language"])
    embeddings = _category(root, categories["embedding"])
    vector_stores = _category(root, categories["vector_store"])
    values = (
        ("Version", version),
        ("Languages", str(len(languages))),
        ("Embedding Providers", str(len(embeddings))),
        ("Vector Stores", str(len(vector_stores))),
        (
            "ADRs",
            str(sum(1 for path in adr_dir.iterdir() if _ADR.fullmatch(path.name))),
        ),
        ("Tests", str(test_count)),
        ("Source Files", str(len(source_files))),
        ("Source Lines", str(source_lines)),
        ("Test Files", str(len(test_files))),
        ("Module Docs", str(len(tuple(module_docs_dir.glob("*.md"))))),
    )
    label_width = max(len(label) for label, _ in values)
    value_width = max(5, *(len(value) for _, value in values))
    table_lines = [
        f"| {'Metric':<{label_width}} | {'Value':>{value_width}} |",
        f"| {'-' * label_width} | {'-' * (value_width - 1)}: |",
        *(
            f"| {label:<{label_width}} | {value:>{value_width}} |"
            for label, value in values
        ),
    ]
    content = template.render(
        version=version,
        version_anchor="v" + re.sub(r"[^a-z0-9]", "", version.lower()),
        table="\n".join(table_lines),
        language_list="\n".join(f"- {name}" for name in languages),
        embedding_list="\n".join(f"- {name}" for name in embeddings),
        vector_store_list="\n".join(f"- {name}" for name in vector_stores),
    )
    return content.rstrip() + "\n"


def generate_metrics(root: Path) -> tuple[Path, bool]:
    """Publish changed bytes atomically and report whether output changed."""
    destination = root / "docs/generated/METRICS.md"
    content = render_metrics(root)
    before = destination.read_text(encoding="utf-8") if destination.exists() else None
    if before == content:
        return destination, False
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=destination.parent, prefix=".metrics-stage-"
    ) as staged:
        staged.write(content)
        staged.flush()
        os.fsync(staged.fileno())
        Path(staged.name).chmod(0o644)
        Path(staged.name).replace(destination)
    return destination, True
