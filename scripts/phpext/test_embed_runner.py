#!/usr/bin/env python3
"""Exercise Apple leak-runner failure reporting with real child processes."""

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import test_embed


class AppleLeakRunnerTests(unittest.TestCase):
    """A diagnostic checkpoint must never turn a failed lifecycle into success."""

    def run_child(self, source, leaks="0 leaks\n", leak_status=0):
        """Replace only Apple tools, retaining real process I/O and exit status."""
        root = Path(__file__).resolve().parents[2]
        (root / "target").mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="phpext-runner-", dir=root / "target") as work:
            command = [sys.executable, "-c", source]
            with patch.object(test_embed, "checked"), patch.object(
                test_embed.subprocess, "run",
                return_value=subprocess.CompletedProcess(["leaks"], leak_status, leaks, ""),
            ):
                test_embed.check_apple_leaks(command, dict(test_embed.os.environ), Path(work))

    def test_early_failure_keeps_stderr(self):
        """An exited child still reports the original lifecycle error."""
        with self.assertRaisesRegex(RuntimeError, "original fatal detail"):
            self.run_child("import sys; print('original fatal detail', file=sys.stderr); sys.exit(1)")

    def test_failed_checkpoint_keeps_status_and_leak_report(self):
        """A completed failed request is inspected but cannot pass even with no leaks."""
        for checkpoint, status in [("failed", 1), ("failed", 0), ("ok", 1)]:
            with self.subTest(checkpoint=checkpoint, status=status):
                with self.assertRaisesRegex(RuntimeError, "lifecycle failure detail") as error:
                    self.run_child(
                        "import sys; print('lifecycle failure detail', file=sys.stderr); "
                        f"print('request isolation {checkpoint}', flush=True); input(); sys.exit({status})"
                    )
                self.assertIn("0 leaks", str(error.exception))

    def test_leak_failure_keeps_lifecycle_diagnostics(self):
        """Leak findings and request failures remain visible together."""
        with self.assertRaisesRegex(RuntimeError, "retained native checkpoint") as error:
            self.run_child(
                "import sys; print('request failure detail', file=sys.stderr); "
                "print('request isolation failed', flush=True); input(); sys.exit(1)",
                leaks="retained native checkpoint\n", leak_status=1,
            )
        self.assertIn("request failure detail", str(error.exception))


if __name__ == "__main__":
    unittest.main()
