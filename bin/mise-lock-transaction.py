# Copyright 2026 FLEXT
"""Publish a Mise lock with its native sidecars from one physical stage.

This bootstrap runs with the Python selected by the staged Mise lock, before
the project's virtual environment exists. It intentionally uses only stdlib.
Its journal and project-scoped mutex recover process interruption on every
platform. Directory fsync is POSIX-only; Windows power-loss durability is not
promised by this transaction.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import subprocess
import sys
import time
import tomllib
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path, PurePosixPath


class MiseLockTransaction:
    """Keep the old lock usable until every new sidecar is published."""

    JOURNAL = "transaction.json"
    NEW_LOCK = "new.lock"
    OLD_LOCK = "old.lock"
    ARTIFACTS = (
("bin/mise", 0o755),
("bin/mise.cmd", 0o644),
("mise.version", 0o644),
)
    MUTEX_TIMEOUT_SECONDS = 600.0
    LOCK = "mise.lock"
    NATIVE_GRAPHS = (("aube", "aube-lock.yaml"), ("uv", "uv.lock"))

    @staticmethod
    @contextmanager
    def _serialized(project: Path) -> Iterator[None]:
        """Serialize all publisher versions on one declared physical mutex."""
        mutex = project / ".mise-lock-transaction.lock"
        if mutex.is_symlink():
            raise ValueError(f"Mise transaction mutex is a symlink: {mutex}")
        flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0)
        descriptor = os.open(mutex, flags, 0o600)
        try:
            observed = os.fstat(descriptor)
            if not stat.S_ISREG(observed.st_mode) or observed.st_nlink != 1:
                raise ValueError(f"Mise transaction mutex is not physical: {mutex}")
            if observed.st_size == 0:
                os.write(descriptor, b"\0")
                os.fsync(descriptor)
            os.lseek(descriptor, 0, os.SEEK_SET)
            if os.name == "nt":
                import msvcrt

                msvcrt.locking(descriptor, msvcrt.LK_LOCK, 1)
            else:
                import fcntl

                deadline = time.monotonic() + MiseLockTransaction.MUTEX_TIMEOUT_SECONDS
                while True:
                    try:
                        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
                        break
                    except OSError:
                        if time.monotonic() >= deadline:
                            raise ValueError(
                                "Mise transaction mutex is held elsewhere for over "
                                f"{MiseLockTransaction.MUTEX_TIMEOUT_SECONDS:.0f}s: {mutex}"
                            ) from None
                        time.sleep(0.2)
            try:
                yield
            finally:
                if os.name == "nt":
                    msvcrt.locking(descriptor, msvcrt.LK_UNLCK, 1)
                else:
                    fcntl.flock(descriptor, fcntl.LOCK_UN)
        finally:
            os.close(descriptor)

    @staticmethod
    def _physical_directory(path: Path) -> None:
        observed = path.lstat()
        if not stat.S_ISDIR(observed.st_mode):
            raise ValueError(f"transaction directory is not physical: {path}")

    @staticmethod
    def _bytes(path: Path) -> bytes | None:
        try:
            observed = path.lstat()
        except FileNotFoundError:
            return None
        if not stat.S_ISREG(observed.st_mode) or observed.st_nlink != 1:
            raise ValueError(f"transaction file is not physical: {path}")
        return path.read_bytes()

    @staticmethod
    def _digest(content: bytes | None) -> str | None:
        return None if content is None else hashlib.sha256(content).hexdigest()

    @staticmethod
    def _sidecar_selector(relative: str) -> PurePosixPath:
        selector = PurePosixPath(relative)
        if (
            selector.is_absolute()
            or selector.as_posix() != relative
            or len(selector.parts) < 4
            or selector.parts[:2] != (".mise", "locks")
            or ".." in selector.parts
        ):
            raise ValueError(f"unsafe {MiseLockTransaction.LOCK} sidecar: {relative}")
        return selector

    @classmethod
    def _sidecar_digest(cls, graph: str, filename: str, annotation: object, root: Path) -> tuple[str, str]:
        """Authenticate one native-graph annotation against its physical sidecar."""
        if not isinstance(annotation, dict):
            raise ValueError(f"{cls.LOCK} {graph} annotation is not a table")
        relative = annotation.get("path")
        digest = annotation.get("digest")
        if not isinstance(relative, str) or not isinstance(digest, str):
            raise ValueError(f"{cls.LOCK} {graph} annotation is incomplete")
        selector = cls._sidecar_selector(relative)
        if not digest.startswith("sha256:"):
            raise ValueError(f"invalid {cls.LOCK} sidecar digest: {relative}")
        cls._reject_symlink_path(root, relative)
        sidecar = root.joinpath(*selector.parts)
        cls._physical_directory(sidecar)
        source = cls._bytes(sidecar / filename)
        if source is None:
            raise ValueError(f"{cls.LOCK} sidecar is absent: {sidecar / filename}")
        actual = hashlib.sha256(source.replace(b"\r\n", b"\n")).hexdigest()
        if actual != digest.removeprefix("sha256:"):
            raise ValueError(f"{cls.LOCK} sidecar digest differs: {sidecar / filename}")
        return relative, cls._tree_digest(sidecar)

    @classmethod
    def _sidecars(cls, content: bytes | None, root: Path) -> dict[str, str]:
        if content is None:
            return {}
        payload = tomllib.loads(content.decode("utf-8"))
        tools = payload.get("tools")
        if not isinstance(tools, dict):
            raise ValueError(f"{cls.LOCK} has no tools table")
        result: dict[str, str] = {}
        for entries in tools.values():
            for entry in entries if isinstance(entries, list) else (entries,):
                if not isinstance(entry, dict):
                    raise ValueError(f"{cls.LOCK} tool entry is not a table")
                for graph, filename in cls.NATIVE_GRAPHS:
                    if entry.get(graph) is not None:
                        relative, tree = cls._sidecar_digest(graph, filename, entry[graph], root)
                        result[relative] = tree
        return result

    @classmethod
    def _previous_sidecars(cls, content: bytes | None, project: Path) -> dict[str, str]:
        """Read the owned graph from Git stage 2 during a lock merge conflict.

        A generated lock with conflict markers is not a TOML declaration. Git's
        unmerged index retains the exact prior lock; its sidecars must still
        validate against the physical checkout before publication can replace
        them. Ordinary malformed locks continue to fail at the TOML parser.
        """
        if content is None or b"<<<<<<< " not in content:
            return cls._sidecars(content, project)
        index = subprocess.run(
            ["git", "-C", str(project), "ls-files", "-u", "--", cls.LOCK],
            check=True,
            capture_output=True,
        ).stdout
        if not any(line.split(b"\t", 1)[0].endswith(b" 2") for line in index.splitlines()):
            raise ValueError(f"conflicted {cls.LOCK} has no Git stage-2 source")
        prior = subprocess.run(
            ["git", "-C", str(project), "show", f":2:{cls.LOCK}"],
            check=True,
            capture_output=True,
        ).stdout
        return cls._sidecars(prior, project)

    @classmethod
    def _tree_digest(cls, root: Path) -> str:
        cls._physical_directory(root)
        checksum = hashlib.sha256()
        for path in sorted(root.rglob("*")):
            observed = path.lstat()
            relative = path.relative_to(root).as_posix().encode()
            if stat.S_ISDIR(observed.st_mode):
                checksum.update(b"D\0" + relative + b"\0")
            elif stat.S_ISREG(observed.st_mode) and observed.st_nlink == 1:
                checksum.update(b"F\0" + relative + b"\0" + path.read_bytes())
            else:
                raise ValueError(f"nonphysical mise sidecar entry: {path}")
        return checksum.hexdigest()

    @staticmethod
    def _sync_directory(path: Path) -> None:
        """Sync POSIX directory metadata; Windows relies on journal recovery.

        Windows process-interruption recovery is covered by the persisted
        journal and same-volume replacements. Python exposes no portable
        directory fsync there, so this makes no power-loss durability claim.
        """
        if os.name == "nt":
            return
        descriptor = os.open(path, os.O_RDONLY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)

    @classmethod
    def _sync_tree(cls, root: Path) -> None:
        """Persist staged payload bytes before publishing the journal."""
        cls._physical_directory(root)
        for path in sorted(root.rglob("*"), reverse=True):
            observed = path.lstat()
            if stat.S_ISDIR(observed.st_mode):
                cls._sync_directory(path)
            elif stat.S_ISREG(observed.st_mode) and observed.st_nlink == 1:
                descriptor = os.open(path, os.O_RDONLY)
                try:
                    os.fsync(descriptor)
                finally:
                    os.close(descriptor)
            else:
                raise ValueError(f"nonphysical Mise stage entry: {path}")
        cls._sync_directory(root)

    @classmethod
    def _write_journal(cls, stage: Path, journal: dict[str, str]) -> None:
        candidate = stage / "transaction.json.new"
        with candidate.open("x", encoding="utf-8") as stream:
            json.dump(journal, stream, sort_keys=True)
            stream.flush()
            os.fsync(stream.fileno())
        candidate.replace(stage / cls.JOURNAL)
        cls._sync_directory(stage)

    @classmethod
    def _read_journal(cls, stage: Path) -> dict[str, str] | None:
        content = cls._bytes(stage / cls.JOURNAL)
        if content is None:
            return None
        payload = json.loads(content)
        if not isinstance(payload, dict) or not all(
            isinstance(key, str) and isinstance(value, str)
            for key, value in payload.items()
        ):
            raise ValueError(f"invalid Mise lock transaction journal: {stage}")
        return payload

    @staticmethod
    def _journal_refs(journal: dict[str, str], name: str) -> dict[str, str]:
        raw = journal.get(name)
        if raw is None:
            raise ValueError(f"Mise lock journal lacks {name}")
        payload = json.loads(raw)
        if not isinstance(payload, dict) or not all(
            isinstance(key, str) and isinstance(value, str)
            for key, value in payload.items()
        ):
            raise ValueError(f"Mise lock journal has invalid {name}")
        for relative in payload:
            MiseLockTransaction._sidecar_selector(relative)
        return payload

    @classmethod
    def _artifact_refs(cls, root: Path) -> dict[str, str]:
        return {
            relative: cls._digest(cls._bytes(root / relative)) or ""
            for relative, _mode in cls.ARTIFACTS
        }

    @classmethod
    def _journal_artifacts(cls, journal: dict[str, str], name: str) -> dict[str, str]:
        raw = journal.get(name)
        if raw is None:
            raise ValueError(f"Mise lock journal lacks {name}")
        payload = json.loads(raw)
        if not isinstance(payload, dict) or not all(
            isinstance(key, str) and isinstance(value, str)
            for key, value in payload.items()
        ):
            raise ValueError(f"Mise lock journal has invalid {name}")
        return payload

    @classmethod
    def _recover_artifacts(cls, project: Path, stage: Path, journal: dict[str, str]) -> None:
        old_refs = cls._journal_artifacts(journal, "old_artifacts")
        new_refs = cls._journal_artifacts(journal, "new_artifacts")
        declared = {relative for relative, _mode in cls.ARTIFACTS}
        if set(old_refs) != declared or set(new_refs) != declared:
            raise ValueError("Mise transaction artifact manifest is incomplete")
        for relative, mode in cls.ARTIFACTS:
            source = stage / "new-artifacts" / relative
            expected = new_refs[relative]
            if cls._digest(cls._bytes(source)) != expected:
                raise ValueError(f"staged Mise artifact changed: {source}")
            target = project / relative
            current = cls._digest(cls._bytes(target))
            if current == expected:
                continue
            if current != (old_refs[relative] or None):
                raise ValueError(f"Mise artifact changed outside transaction: {target}")
            pending = stage / "pending-artifacts" / relative
            cls._ensure_parent(stage, pending)
            shutil.copyfile(source, pending)
            pending.chmod(mode)
            descriptor = os.open(pending, os.O_RDONLY)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
            cls._ensure_parent(project, target)
            pending.replace(target)
            cls._sync_directory(target.parent)

    @classmethod
    def _require_roots(cls, project: Path, stage: Path) -> None:
        cls._physical_directory(project)
        cls._physical_directory(stage)
        if stage.parent != project.parent or stage == project:
            raise ValueError("Mise lock stage must be a sibling of its destination")
        if stage.stat().st_dev != project.stat().st_dev:
            raise ValueError("Mise lock stage is not on the destination filesystem")
        if not stage.name.startswith(f".{project.name}.mise-lock-stage."):
            raise ValueError(f"unexpected Mise lock transaction stage: {stage}")

    @staticmethod
    def _reject_symlink_path(project: Path, relative: str) -> None:
        cursor = project
        for part in PurePosixPath(relative).parts:
            cursor /= part
            if cursor.is_symlink():
                raise ValueError(f"Mise sidecar path contains a symlink: {cursor}")

    @classmethod
    def _ensure_parent(cls, root: Path, target: Path) -> None:
        """Materialize physical parents and durably record each directory entry."""
        if not target.is_relative_to(root):
            raise ValueError(f"Mise sidecar escapes transaction root: {target}")
        missing: list[Path] = []
        cursor = target.parent
        while cursor != root:
            missing.append(cursor)
            cursor = cursor.parent
        for directory in reversed(missing):
            if directory.exists():
                cls._physical_directory(directory)
            else:
                directory.mkdir()
                cls._sync_directory(directory.parent)

    @classmethod
    def _retire_stage(cls, stage: Path) -> None:
        """Move a completed journal out of the recovery scan before deleting it."""
        retired = stage.with_name(
            stage.name.replace(".mise-lock-stage.", ".mise-lock-cleanup.", 1),
        )
        if retired.exists() or retired.is_symlink():
            raise ValueError(f"Mise cleanup target already exists: {retired}")
        stage.rename(retired)
        cls._sync_directory(stage.parent)
        shutil.rmtree(retired)

    @classmethod
    def _move(cls, source: Path, destination: Path, root: Path) -> None:
        """Rename one physical entry under ``root`` and persist both directories."""
        cls._ensure_parent(root, destination)
        source.rename(destination)
        cls._sync_directory(destination.parent)
        cls._sync_directory(source.parent)

    @staticmethod
    def _write_durable(path: Path, content: bytes) -> None:
        with path.open("xb") as stream:
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())

    @classmethod
    def _recovery_state(
        cls,
        project: Path,
        stage: Path,
    ) -> tuple[dict[str, str], bytes | None, bytes, dict[str, str], dict[str, str]]:
        """Authenticate a stage journal and the lock copies it recorded."""
        cls._require_roots(project, stage)
        journal = cls._read_journal(stage)
        if journal is None:
            raise ValueError(f"uncommitted Mise stage has no recovery journal: {stage}")
        if journal.get("project") != str(project):
            raise ValueError(f"Mise lock journal belongs to another project: {stage}")
        old = cls._bytes(stage / cls.OLD_LOCK)
        new = cls._bytes(stage / cls.NEW_LOCK)
        if new is None:
            raise ValueError(f"Mise lock journal lost new lock: {stage}")
        if cls._digest(old) != (journal.get("old") or None) or cls._digest(new) != journal.get("new"):
            raise ValueError(f"Mise lock journal digest changed: {stage}")
        old_refs = cls._journal_refs(journal, "old_refs")
        new_refs = cls._journal_refs(journal, "new_refs")
        for relative in old_refs | new_refs:
            cls._reject_symlink_path(project, relative)
        return journal, old, new, old_refs, new_refs

    @classmethod
    def _undo_sidecar(
        cls,
        project: Path,
        stage: Path,
        relative: str,
        expected: str,
        old_refs: dict[str, str],
    ) -> None:
        """Restore one sidecar of a publication whose lock never committed."""
        destination = project / relative
        backup = stage / "old-sidecars" / relative
        if (
            destination.exists()
            and cls._tree_digest(destination) == expected
            and expected != old_refs.get(relative)
        ):
            cls._move(destination, stage / "abandoned-sidecars" / relative, stage)
        if backup.exists():
            if destination.exists() or cls._tree_digest(backup) != old_refs[relative]:
                raise ValueError(f"old Mise sidecar changed during recovery: {backup}")
            cls._move(backup, destination, project)
        elif relative in old_refs:
            if not destination.exists() or cls._tree_digest(destination) != old_refs[relative]:
                raise ValueError(f"old Mise sidecar missing during recovery: {destination}")
        elif destination.exists():
            raise ValueError(f"unowned Mise sidecar changed during recovery: {destination}")

    @classmethod
    def _finish_sidecars(
        cls,
        project: Path,
        stage: Path,
        old_refs: dict[str, str],
        new_refs: dict[str, str],
    ) -> None:
        """Verify the committed sidecars and retire the ones the lock dropped."""
        for relative, expected in new_refs.items():
            destination = project / relative
            if not destination.exists() or cls._tree_digest(destination) != expected:
                raise ValueError(f"committed Mise sidecar differs: {destination}")
        for relative, expected in old_refs.items():
            if relative in new_refs:
                continue
            destination = project / relative
            retired = stage / "retired-sidecars" / relative
            if destination.exists():
                if cls._tree_digest(destination) != expected:
                    raise ValueError(f"stale sidecar changed during recovery: {destination}")
                cls._move(destination, retired, stage)
            elif not retired.exists():
                raise ValueError(f"stale sidecar disappeared during recovery: {destination}")

    @classmethod
    def recover(cls, project: Path, stage: Path) -> None:
        """Finish or undo a prior interrupted publication by its lock commit point.

        A live lock equal to the new one rolls forward. When ``upg`` resolves
        another Mise release but the bumped lock is byte-identical to the
        committed one, old and new coincide and the lock rename commits
        nothing; the journaled pin and launchers are then the publication's
        only change, fully staged and digest-verified before the journal was
        written, so they still move forward instead of being discarded.
        """
        journal, old, new, old_refs, new_refs = cls._recovery_state(project, stage)
        live = cls._bytes(project / cls.LOCK)
        if live == new:
            cls._finish_sidecars(project, stage, old_refs, new_refs)
            if "new_artifacts" in journal:
                cls._recover_artifacts(project, stage, journal)
        elif live == old:
            for relative, expected in new_refs.items():
                cls._undo_sidecar(project, stage, relative, expected, old_refs)
        else:
            raise ValueError(f"Mise lock changed outside transaction: {project / cls.LOCK}")
        cls._retire_stage(stage)

    @classmethod
    def _recover_prior_stages(cls, project: Path, stage: Path) -> None:
        """Settle every earlier publication before this one starts."""
        for prior in sorted(project.parent.glob(f".{project.name}.mise-lock-stage.*")):
            if prior != stage:
                cls.recover(project, prior)
        for retired in sorted(project.parent.glob(f".{project.name}.mise-lock-cleanup.*")):
            cls._physical_directory(retired)
            journal = cls._read_journal(retired)
            if journal is None or journal.get("project") != str(project):
                raise ValueError(f"unowned Mise cleanup directory: {retired}")
            shutil.rmtree(retired)

    @classmethod
    def _stage_artifacts(cls, project: Path, stage: Path) -> tuple[dict[str, str], dict[str, str]]:
        """Copy the staged launcher/pin set into the journal-owned stage area."""
        artifact_stage = stage / "artifacts"
        if not (artifact_stage.exists() or artifact_stage.is_symlink()):
            return {}, {}
        cls._physical_directory(artifact_stage)
        new_artifacts = cls._artifact_refs(artifact_stage)
        if any(not value for value in new_artifacts.values()):
            raise ValueError("staged Mise launcher/pin set is incomplete")
        old_artifacts = cls._artifact_refs(project)
        for relative, _mode in cls.ARTIFACTS:
            destination = stage / "new-artifacts" / relative
            cls._ensure_parent(stage, destination)
            shutil.copyfile(artifact_stage / relative, destination)
        return old_artifacts, new_artifacts

    @classmethod
    def _check_targets(
        cls,
        project: Path,
        old_refs: dict[str, str],
        new_refs: dict[str, str],
    ) -> None:
        """Refuse a publication whose sidecar targets changed outside it."""
        for relative in old_refs | new_refs:
            cls._reject_symlink_path(project, relative)
        for relative, expected in new_refs.items():
            destination = project / relative
            if not destination.exists():
                continue
            # A new declaration cannot claim an existing, unowned path:
            # replacing it would discard data that the old lock never
            # entrusted to this transaction, and rollback could not
            # restore it from the old lock's sidecar manifest.
            if relative not in old_refs:
                raise ValueError(f"unowned Mise sidecar occupies target: {destination}")
            actual = cls._tree_digest(destination)
            if actual != expected and actual != old_refs[relative]:
                raise ValueError(f"Mise sidecar changed outside transaction: {destination}")

    @classmethod
    def _place_sidecars(cls, project: Path, stage: Path, new_refs: dict[str, str]) -> None:
        """Move each staged sidecar into place, parking the one it replaces."""
        for relative, expected in new_refs.items():
            destination = project / relative
            if destination.exists():
                if cls._tree_digest(destination) == expected:
                    continue
                cls._move(destination, stage / "old-sidecars" / relative, stage)
            cls._move(stage / relative, destination, project)

    @classmethod
    def publish(cls, project: Path, stage: Path) -> None:
        """Publish sidecars first and make the lock rename the commit point."""
        cls._require_roots(project, stage)
        cls._recover_prior_stages(project, stage)
        old = cls._bytes(project / cls.LOCK)
        new = cls._bytes(stage / cls.LOCK)
        if new is None:
            raise ValueError(f"staged {cls.LOCK} is absent: {stage}")
        old_refs = cls._previous_sidecars(old, project)
        new_refs = cls._sidecars(new, stage)
        old_artifacts, new_artifacts = cls._stage_artifacts(project, stage)
        cls._sync_tree(stage)
        cls._check_targets(project, old_refs, new_refs)
        if old is not None:
            cls._write_durable(stage / cls.OLD_LOCK, old)
        cls._write_durable(stage / cls.NEW_LOCK, new)
        cls._sync_directory(stage)
        journal = {
            "project": str(project),
            "old": cls._digest(old) or "",
            "new": cls._digest(new) or "",
            "old_refs": json.dumps(old_refs, sort_keys=True),
            "new_refs": json.dumps(new_refs, sort_keys=True),
        }
        if new_artifacts:
            journal["old_artifacts"] = json.dumps(old_artifacts, sort_keys=True)
            journal["new_artifacts"] = json.dumps(new_artifacts, sort_keys=True)
        cls._write_journal(stage, journal)
        cls._place_sidecars(project, stage, new_refs)
        (stage / cls.LOCK).replace(project / cls.LOCK)
        cls._sync_directory(project)
        cls.recover(project, stage)

    @classmethod
    def main(cls, arguments: list[str]) -> int:
        if len(arguments) != 3 or arguments[0] not in {"publish", "recover"}:
            raise ValueError("usage: mise-lock-transaction.py (publish|recover) PROJECT STAGE")
        project = Path(arguments[1]).absolute()
        stage = Path(arguments[2]).absolute()
        with cls._serialized(project):
            if arguments[0] == "publish":
                cls.publish(project, stage)
            elif stage.exists() or stage.is_symlink():
                cls.recover(project, stage)
        return 0


if __name__ == "__main__":
    raise SystemExit(MiseLockTransaction.main(sys.argv[1:]))
