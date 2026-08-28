"""Lib Tests Test Gitops.

Copyright (c) 2025 MCB Contributors. All rights reserved.
SPDX-License-Identifier: MIT
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from lib.gitops import (
    GitOpsTarget,
    _cached_render,
    _render_cache_key,
    analyze,
    discover_targets,
    summarize,
)

from ._utilities.matchers import tm

SCRIPTS = Path(__file__).resolve().parents[2]


def _write_k8s_file(root: Path, rel_path: str, content: str) -> Path:
    path = root / rel_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    return path


def test_readme_only_k8s_tree_is_clean_skip(temp_dir: Path) -> None:
    _write_k8s_file(temp_dir, "k8s/README.md", "# placeholder\n")

    summary_result = summarize(temp_dir / "k8s")

    tm.ok(summary_result)
    summary = summary_result.unwrap()
    assert summary.status == "SKIP"
    assert summary.targets == []
    assert "no Helm or Kustomize targets" in summary.message


def test_discovers_helm_and_kustomize_targets(temp_dir: Path) -> None:
    _write_k8s_file(temp_dir, "k8s/chart/Chart.yaml", "apiVersion: v2\nname: sample\n")
    _write_k8s_file(temp_dir, "k8s/overlay/kustomization.yaml", "resources: []\n")

    targets = discover_targets(temp_dir / "k8s")

    assert [(target.kind, target.path.name) for target in targets] == [
        ("helm", "chart"),
        ("kustomize", "overlay"),
    ]


def test_policy_issues_reuse_qlty_report_model(temp_dir: Path) -> None:
    _write_k8s_file(
        temp_dir,
        "k8s/workload.yaml",
        """\
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sample
spec:
  template:
    spec:
      containers:
        - name: app
          image: nginx:latest
""",
    )

    report_result = analyze(temp_dir / "k8s")

    tm.ok(report_result)
    report = report_result.unwrap()
    assert report.total_issues == 1
    assert report.issues[0].rule_id == "gitops:no-latest-image"
    assert report.by_category["gitops"] == 1


def test_policy_issue_line_points_to_image_key_not_first_matching_value(
    temp_dir: Path,
) -> None:
    _write_k8s_file(
        temp_dir,
        "k8s/workload.yaml",
        """\
apiVersion: v1
kind: Pod
metadata:
  name: latest
  annotations:
    copied-image: busybox:latest
spec:
  containers:
    - name: app
      image: busybox:latest
""",
    )

    report_result = analyze(temp_dir / "k8s")

    tm.ok(report_result)
    report = report_result.unwrap()
    assert report.total_issues == 1
    assert report.issues[0].start_line == 10


def test_command_skips_without_using_cluster_clis() -> None:
    command = SCRIPTS / "check" / "gitops.py"

    result = subprocess.run(
        [sys.executable, str(command), "--root", str(SCRIPTS.parents[0])],
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    assert "GITOPS SKIP" in result.stdout
    combined = result.stdout + result.stderr
    assert "kubectl" not in combined
    assert "vault " not in combined
    assert "argocd " not in combined


def test_command_fails_on_latest_image_policy_issue(temp_dir: Path) -> None:
    command = SCRIPTS / "check" / "gitops.py"
    _write_k8s_file(
        temp_dir,
        "k8s/pod.yaml",
        """\
apiVersion: v1
kind: Pod
metadata:
  name: latest
spec:
  containers:
    - name: app
      image: busybox:latest
""",
    )

    result = subprocess.run(
        [sys.executable, str(command), "--root", str(temp_dir)],
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 1, result.stdout + result.stderr
    assert "gitops:no-latest-image" in result.stdout


def test_render_cache_key_is_stable_for_same_inputs(temp_dir: Path) -> None:
    _write_k8s_file(temp_dir, "k8s/chart/Chart.yaml", "apiVersion: v2\nname: sample\n")
    target = GitOpsTarget(kind="helm", path=temp_dir / "k8s" / "chart")

    assert _render_cache_key(target) == _render_cache_key(target)


def test_render_cache_key_changes_when_input_changes(temp_dir: Path) -> None:
    chart_dir = temp_dir / "k8s" / "chart"
    _write_k8s_file(temp_dir, "k8s/chart/Chart.yaml", "apiVersion: v2\nname: sample\n")
    target = GitOpsTarget(kind="helm", path=chart_dir)
    before = _render_cache_key(target)
    (chart_dir / "values.yaml").write_text("foo: bar\n", encoding="utf-8")
    after = _render_cache_key(target)

    assert before != after


def test_cached_render_writes_and_reuses_cache(
    temp_dir: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    chart_dir = temp_dir / "k8s" / "chart"
    _write_k8s_file(temp_dir, "k8s/chart/Chart.yaml", "apiVersion: v2\nname: sample\n")
    target = GitOpsTarget(kind="helm", path=chart_dir)
    rendered = "apiVersion: v1\nkind: Pod\nmetadata:\n  name: test\n"

    calls: list[list[str]] = []

    def fake_run(cmd: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
        calls.append(cmd)
        return subprocess.CompletedProcess(
            args=cmd, returncode=0, stdout=rendered, stderr=""
        )

    monkeypatch.setattr(subprocess, "run", fake_run)
    first = _cached_render(target)
    second = _cached_render(target)

    assert first == rendered
    assert second == rendered
    assert len(calls) == 1, "cache should prevent a second render"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
