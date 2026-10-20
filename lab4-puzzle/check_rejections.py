"""Compile the invalid callers and verify the intended diagnostics."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--crate-dir", type=Path, default=Path(__file__).resolve().parent)
parser.add_argument("--filter", default="", help="Run matching fixture names only")
args = parser.parse_args()
fixture_root = Path(__file__).resolve().parent
crate_root = args.crate_dir.resolve()
package_name = "lab4-puzzle"
cargo_command = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")
fixtures = sorted(p for p in (fixture_root / "ui").glob("*.rs") if args.filter in p.stem)
if not fixtures:
    raise SystemExit("No fixtures selected; check --filter.")
target = crate_root / "target/rejection-checks"
logs = crate_root / "target/rejection-logs"
logs.mkdir(parents=True, exist_ok=True)
child_env = os.environ.copy()
child_env["CARGO_TARGET_DIR"] = str(target)
failed = []
with tempfile.TemporaryDirectory(prefix="rust-rejection-") as directory:
    probe = Path(directory)
    (probe / "src").mkdir()
    (probe / "Cargo.toml").write_text(
        '[package]\nname="rejection-probe"\nversion="0.1.0"\nedition="2024"\n'
        '[workspace]\n[dependencies]\n'
        + package_name + "={path=" + json.dumps(str(crate_root)) + "}\n"
    )
    for fixture in fixtures:
        text = fixture.read_text()
        expected = next(line.removeprefix("// EXPECT: ") for line in text.splitlines() if line.startswith("// EXPECT: "))
        (probe / "src/main.rs").write_text(text)
        result = subprocess.run(
            [cargo_command, "+nightly-2026-07-22", "check", "--manifest-path", str(probe / "Cargo.toml"), "--offline"],
            capture_output=True, text=True, env=child_env, timeout=60
        )
        output = result.stdout + result.stderr
        (logs / (fixture.stem + ".log")).write_text(output)
        ok = result.returncode != 0 and expected in output and "proc-macro derive panicked" not in output
        print(("PASS" if ok else "FAIL") + ": " + fixture.name + " -> " + expected, flush=True)
        if not ok:
            failed.append(fixture.name)
            print(output[-3500:], flush=True)
if failed:
    raise SystemExit("Unexpected results: " + ", ".join(failed))
print(f"{len(fixtures)} intended compiler rejections verified.")
