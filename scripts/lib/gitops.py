"""Lib Gitops.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""
from __future__ import annotations

import hashlib
import subprocess
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from flext_core import FlextResult
from kubernetes_validate import validate_resource
from qlty.model import SarifIssue, Severity
from qlty.report import AnalysisReport, analyze_issues
from ruamel.yaml import YAML
from ruamel.yaml.comments import CommentedMap, CommentedSeq
from ruamel.yaml.error import YAMLError

from lib.core import get_logger, r

logger = get_logger(__name__)

KUSTOMIZE_FILES = frozenset(
    {
        "kustomization.yaml",
        "kustomization.yml",
        "Kustomization",
    }
)
YAML_SUFFIXES = frozenset({".yaml", ".yml"})
CACHE_DIR = Path(".gitops-cache")
DEFAULT_KUBE_VERSION = "1.32.0"


@dataclass(frozen=True, slots=True)
class GitOpsTarget:
    """A Helm or Kustomize render target."""

    kind: str
    path: Path


@dataclass(frozen=True, slots=True)
class GitOpsSummary:
    """Result of the lightweight GitOps discovery pass."""

    status: str
    message: str
    targets: list[GitOpsTarget]
    report: AnalysisReport


@dataclass(frozen=True, slots=True)
class ImageReference:
    """Container image value with its YAML source line."""

    value: str
    line: int


YamlNode = CommentedMap | CommentedSeq | str | int | float | bool | None


def discover_targets(root: Path) -> list[GitOpsTarget]:
    """Discover Helm and Kustomize render targets below ``root``."""

    if not root.exists():
        return []

    targets: list[GitOpsTarget] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        if path.name == "Chart.yaml":
            targets.append(GitOpsTarget(kind="helm", path=path.parent))
            continue
        if path.name in KUSTOMIZE_FILES:
            targets.append(GitOpsTarget(kind="kustomize", path=path.parent))

    seen: set[tuple[str, Path]] = set()
    unique: list[GitOpsTarget] = []
    for target in targets:
        key = (target.kind, target.path)
        if key in seen:
            continue
        seen.add(key)
        unique.append(target)
    return unique


def summarize(root: Path) -> FlextResult[GitOpsSummary]:
    """Return a discovery summary for GitOps targets below ``root``."""

    targets = discover_targets(root)
    report_result = analyze(root)
    if report_result.failure:
        return FlextResult[GitOpsSummary].fail(report_result.error or "gitops analysis failed")
    report = report_result.unwrap()

    if report.total_issues:
        return FlextResult[GitOpsSummary].ok(
            GitOpsSummary(
                status="FAIL",
                message=f"{root}: {report.total_issues} GitOps policy issue(s)",
                targets=targets,
                report=report,
            )
        )
    if not targets:
        return FlextResult[GitOpsSummary].ok(
            GitOpsSummary(
                status="SKIP",
                message=f"{root}: no Helm or Kustomize targets found",
                targets=[],
                report=report,
            )
        )
    return FlextResult[GitOpsSummary].ok(
        GitOpsSummary(
            status="OK",
            message=f"{root}: discovered {len(targets)} GitOps target(s)",
            targets=targets,
            report=report,
        )
    )


def analyze(root: Path) -> r[AnalysisReport]:
    """Analyze GitOps source manifests through the existing qlty report model."""

    issues = policy_issues(root) + rendered_issues(root)
    return analyze_issues(issues)


def policy_issues(root: Path) -> list[SarifIssue]:
    """Return native GitOps policy issues discovered in source manifests."""

    issues: list[SarifIssue] = []
    for path in _yaml_files(root):
        try:
            documents = _load_yaml_documents(path)
        except YAMLError as exc:
            issues.append(
                _issue(
                    "gitops:yaml-parse",
                    f"YAML parse error: {exc}",
                    path,
                    1,
                )
            )
            continue
        for document in documents:
            for image in _image_references(document):
                if image.value.endswith(":latest"):
                    issues.append(
                        _issue(
                            "gitops:no-latest-image",
                            f"Container image must not use the mutable latest tag: {image.value}",
                            path,
                            image.line,
                        )
                    )
    return issues


def rendered_issues(root: Path, threads: int = 4) -> list[SarifIssue]:
    """Render Helm/Kustomize targets and validate emitted manifests."""

    targets = discover_targets(root)
    if not targets:
        return []

    issues: list[SarifIssue] = []
    with ThreadPoolExecutor(max_workers=threads) as pool:
        for target_issues in pool.map(_render_and_validate, targets):
            issues.extend(target_issues)
    return issues


def _render_and_validate(target: GitOpsTarget) -> list[SarifIssue]:
    """Render a single target and run schema validation on the output."""

    rendered = _cached_render(target)
    if rendered is None:
        return []
    issues: list[SarifIssue] = []
    parser = YAML(typ="safe")
    try:
        documents = list(parser.load_all(rendered))
    except YAMLError as exc:
        return [
            _issue(
                "gitops:render-yaml-parse",
                f"Rendered YAML parse error: {exc}",
                target.path,
                1,
            )
        ]

    for idx, document in enumerate(documents):
        if not isinstance(document, dict):
            continue
        line = document.get("__line__", idx + 1)
        try:
            validate_resource(
                document,
                filename=str(target.path),
                version=DEFAULT_KUBE_VERSION,
                strict=False,
                quiet=True,
                no_warn=True,
            )
        except Exception as exc:  # kubernetes_validate raises various exceptions
            issues.append(
                _issue(
                    "gitops:schema-invalid",
                    f"Schema validation failed: {exc}",
                    target.path,
                    line,
                )
            )
    return issues


def _cached_render(target: GitOpsTarget) -> str | None:
    """Render a target, caching the result by input content hash."""

    cache_key = _render_cache_key(target)
    cache_path = CACHE_DIR / f"{cache_key}.yaml"
    if cache_path.exists():
        return cache_path.read_text(encoding="utf-8")

    rendered = _render_target(target)
    if rendered is None:
        return None
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    cache_path.write_text(rendered, encoding="utf-8")
    return rendered


def _render_cache_key(target: GitOpsTarget) -> str:
    """Return a stable hash for the target's inputs."""

    hasher = hashlib.sha256()
    hasher.update(target.kind.encode())
    hasher.update(str(target.path).encode())
    for path in sorted(target.path.rglob("*")):
        if path.is_file():
            hasher.update(path.read_bytes())
    return hasher.hexdigest()


def _render_target(target: GitOpsTarget) -> str | None:
    """Run helm template or kustomize build for a target."""

    if target.kind == "helm":
        cmd = ["helm", "template", str(target.path)]
    elif target.kind == "kustomize":
        cmd = ["kustomize", "build", str(target.path)]
    else:
        return None

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            check=False,
            timeout=60,
        )
    except FileNotFoundError:
        logger.warning(f"{target.kind} CLI not installed; skipping {target.path}")
        return None
    except subprocess.TimeoutExpired:
        logger.warning(f"{target.kind} render timed out for {target.path}")
        return None

    if result.returncode != 0:
        logger.warning(f"{target.kind} render failed for {target.path}: {result.stderr.strip()}")
        return None
    return result.stdout


def _yaml_files(root: Path) -> list[Path]:
    if not root.exists():
        return []
    return sorted(path for path in root.rglob("*") if path.is_file() and path.suffix in YAML_SUFFIXES)


def _load_yaml_documents(path: Path) -> list[YamlNode]:
    parser = YAML(typ="rt")
    return cast("list[YamlNode]", list(parser.load_all(path)))


def _image_references(node: YamlNode) -> list[ImageReference]:
    references: list[ImageReference] = []
    if isinstance(node, CommentedMap):
        image = node.get("image")
        if isinstance(image, str):
            references.append(ImageReference(value=image, line=_line_for_key(node, "image")))
        for child in node.values():
            references.extend(_image_references(cast("YamlNode", child)))
    elif isinstance(node, CommentedSeq):
        for child in node:
            references.extend(_image_references(cast("YamlNode", child)))
    return references


def _line_for_key(node: CommentedMap, key: str) -> int:
    line, _column = cast("tuple[int, int]", node.lc.key(key))
    return line + 1


def _issue(rule_id: str, message: str, path: Path, line: int) -> SarifIssue:
    return SarifIssue(
        rule_id=rule_id,
        level=Severity.ERROR,
        message=message,
        file_path=str(path),
        start_line=line,
        category="gitops",
    )
