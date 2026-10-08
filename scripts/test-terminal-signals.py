"""Verify native and npm TUI cleanup with a PTY and empty synthetic logs."""
import os
from pathlib import Path
import pty
import select
import shutil
import signal
import subprocess
import sys
import tempfile
import termios
import time
import tty


def check(command, signum, root):
    master, slave = pty.openpty()
    # On macOS, the first raw-mode round trip adds EXTPROC bookkeeping.
    # Settle the newly created PTY before taking an exact restoration baseline.
    initial = termios.tcgetattr(slave)
    tty.setraw(slave)
    termios.tcsetattr(slave, termios.TCSANOW, initial)
    before = termios.tcgetattr(slave)
    proc = subprocess.Popen(
        command + ["--codex-dir", str(root / "codex"), "--claude-dir", str(root / "claude"),
                   "--cache", str(root / "cache.sqlite3")],
        stdin=slave, stdout=slave, stderr=slave,
    )
    output = b""
    deadline = time.monotonic() + 10
    try:
        # Wait for the first frame to hide the cursor as well as enabling mouse
        # capture; otherwise a startup signal need not emit ShowCursor at all.
        while b"?1006h" not in output or b"?25l" not in output:
            assert time.monotonic() < deadline, "TUI did not finish terminal setup"
            assert proc.poll() is None, output
            if select.select([master], [], [], 0.1)[0]:
                output += os.read(master, 65536)
        assert termios.tcgetattr(slave) != before, "TUI did not enable raw mode"
        proc.send_signal(signum)
        assert proc.wait(timeout=5) == 0, "signal did not cause graceful shutdown"
        while select.select([master], [], [], 0.1)[0]:
            output += os.read(master, 65536)
        assert termios.tcgetattr(slave) == before, "termios was not restored"
        for code in [b"?1006l", b"?1015l", b"?1003l", b"?1002l", b"?1000l", b"?1049l", b"?25h"]:
            assert code in output, f"missing terminal cleanup: {code!r}"
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)
        os.close(slave)


binary = Path(sys.argv[1]).resolve()
targets = {("darwin", "arm64"): "aarch64-apple-darwin",
           ("darwin", "x86_64"): "x86_64-apple-darwin",
           ("linux", "aarch64"): "aarch64-unknown-linux-gnu",
           ("linux", "x86_64"): "x86_64-unknown-linux-gnu"}
with tempfile.TemporaryDirectory(prefix="token-speed-signal-") as temporary:
    root = Path(temporary)
    (root / "codex").mkdir()
    (root / "claude").mkdir()
    (root / "bin").mkdir()
    launcher = root / "bin/token-speed.cjs"
    shutil.copyfile(Path(__file__).resolve().parent.parent / "npm/bin/token-speed.cjs", launcher)
    vendor = root / "vendor" / targets[(sys.platform, os.uname().machine)]
    vendor.mkdir(parents=True)
    (vendor / "token-speed").symlink_to(binary)
    for command in [[str(binary)], ["node", str(launcher)]]:
        for signum in [signal.SIGINT, signal.SIGTERM, signal.SIGHUP]:
            check(command, signum, root)
            print(f"{Path(command[0]).name}: {signum.name} restored terminal")
