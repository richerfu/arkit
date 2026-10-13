#!/usr/bin/env python3
"""Run the production PTY and terminal input tests on a Unix host."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


class TerminalHostTests:
    def __init__(self, root: Path):
        self.root = root
        manifest = tomllib.loads((root / "Cargo.toml").read_text())
        self.libc_version = manifest["workspace"]["dependencies"]["libc"]

    @staticmethod
    def quote(value: str | Path) -> str:
        return json.dumps(str(value), ensure_ascii=False)

    def run(self, toolchain: str | None, offline: bool, target_dir: Path | None) -> int:
        with tempfile.TemporaryDirectory(prefix="arkit-terminal-host-") as directory:
            project = Path(directory)
            (project / "src").mkdir()
            (project / "tests").mkdir()
            # Reuse the workspace's resolved versions, including yanked
            # versions still intentionally pinned by its lockfile.
            shutil.copyfile(self.root / "Cargo.lock", project / "Cargo.lock")
            (project / "Cargo.toml").write_text(
                '[package]\nname = "arkit-terminal-host-tests"\n'
                'version = "0.0.0"\nedition = "2021"\n'
                '[dependencies]\n'
                f'libc = {self.quote(self.libc_version)}\n'
                'arkit_prelude = { path = '
                f'{self.quote(self.root / "crates/arkit_prelude")} }}\n'
                '[lints.rust]\nunsafe_op_in_unsafe_fn = "deny"\n'
                '[lints.clippy]\nundocumented_unsafe_blocks = "deny"\n'
            )
            (project / "src/lib.rs").write_text(
                '#[path = '
                f'{self.quote(self.root / "examples/terminal/src/local_pty.rs")}]\n'
                'pub mod local_pty;\n'
            )
            contract = self.root / "crates/arkit_terminal/tests/input_contract.rs"
            source = contract.read_text()
            for module in ("config", "error", "frame", "input"):
                source = source.replace(
                    f'#[path = "../src/{module}.rs"]',
                    '#[path = '
                    f'{self.quote(self.root / "crates/arkit_terminal/src" / (module + ".rs"))}]',
                )
            (project / "tests/input_contract.rs").write_text(source)
            command = ["cargo"]
            if toolchain:
                command.append("+" + toolchain)
            command.extend(["test", "--manifest-path", str(project / "Cargo.toml")])
            if offline:
                command.append("--offline")
            if target_dir:
                command.extend(["--target-dir", str(target_dir.resolve())])
            return subprocess.run(command, check=False).returncode


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--toolchain", help="Rust toolchain, for example 1.97.1")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--target-dir", type=Path, help="Reusable Cargo build cache")
    arguments = parser.parse_args()
    return TerminalHostTests(Path(__file__).resolve().parents[1]).run(
        arguments.toolchain, arguments.offline, arguments.target_dir
    )


if __name__ == "__main__":
    raise SystemExit(main())
