from __future__ import annotations

import os
import stat
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SETUP_CI = ROOT / ".github" / "setup-ci.sh"


def _write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


def test_sccache_bootstrap_disables_wrapper_only_for_install() -> None:
    with tempfile.TemporaryDirectory() as temp_dir:
        _run_sccache_bootstrap_regression(Path(temp_dir))


def _run_sccache_bootstrap_regression(tmp_path: Path) -> None:
    bin_dir = tmp_path / "bin"
    log_file = tmp_path / "commands.log"
    bin_dir.mkdir()

    _write_executable(bin_dir / "uname", "#!/bin/sh\nprintf 'Darwin\\n'\n")
    _write_executable(bin_dir / "protoc", "#!/bin/sh\nexit 0\n")
    _write_executable(bin_dir / "find", "#!/bin/sh\nprintf '/usr/local/lib/libonnxruntime.dylib\\n'\n")
    _write_executable(bin_dir / "grep", "#!/bin/sh\n/bin/grep \"$@\"\n")
    _write_executable(
        bin_dir / "cargo",
        "#!/bin/sh\n"
        "printf 'cargo wrapper=%s args=%s\\n' \"${RUSTC_WRAPPER-<unset>}\" \"$*\" >>\"$COMMAND_LOG\"\n"
        "/bin/cat >\"$FAKE_BIN/sccache\" <<'EOF'\n"
        "#!/bin/sh\n"
        "printf 'sccache wrapper=%s args=%s\\n' \"${RUSTC_WRAPPER-<unset>}\" \"$*\" >>\"$COMMAND_LOG\"\n"
        "exit 0\n"
        "EOF\n"
        "/bin/chmod +x \"$FAKE_BIN/sccache\"\n",
    )

    env = os.environ.copy()
    env.update(
        {
            "COMMAND_LOG": str(log_file),
            "FAKE_BIN": str(bin_dir),
            "PATH": str(bin_dir),
            "RUSTC_WRAPPER": "sccache",
        }
    )

    result = subprocess.run(
        ["/bin/bash", str(SETUP_CI)],
        cwd=ROOT,
        env=env,
        check=False,
        capture_output=True,
        text=True,
        timeout=5,
    )

    assert result.returncode == 0, result.stdout + result.stderr
    commands = log_file.read_text(encoding="utf-8").splitlines()
    assert commands == [
        "cargo wrapper= args=install sccache --locked --quiet",
        "sccache wrapper=sccache args=rustc -vV",
    ], commands


if __name__ == "__main__":
    test_sccache_bootstrap_disables_wrapper_only_for_install()
