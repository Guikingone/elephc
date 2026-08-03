from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).resolve().parents[1] / "benchmark_magician.py"
SPEC = importlib.util.spec_from_file_location("benchmark_magician", SCRIPT)
assert SPEC is not None
assert SPEC.loader is not None
benchmark_magician = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = benchmark_magician
SPEC.loader.exec_module(benchmark_magician)


class ManagedNativeProjectTests(unittest.TestCase):
    def test_prepare_installs_the_locked_benchmark_project(self) -> None:
        binary = Path("/tmp/test-elephc")
        with (
            mock.patch.object(benchmark_magician, "elephc_bin", return_value=binary),
            mock.patch.object(benchmark_magician, "run_process") as run_process,
        ):
            benchmark_magician.prepare_managed_native_dependencies()

        project = benchmark_magician.benchmark_project_root()
        run_process.assert_called_once_with(
            [
                str(binary),
                "native",
                "install",
                "--locked",
                "--manifest-path",
                str(project / "elephc.toml"),
            ],
            project,
        )

    def test_stage_places_manifest_and_lock_above_case_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            project = Path(temp_dir)
            case = project / "arithmetic_loop"
            case.mkdir()

            benchmark_magician.stage_native_project(project)

            for filename in benchmark_magician.NATIVE_PROJECT_FILES:
                staged = project / filename
                self.assertEqual(
                    staged.read_bytes(),
                    (benchmark_magician.benchmark_project_root() / filename).read_bytes(),
                )
                self.assertEqual(staged.parent, case.parent)


if __name__ == "__main__":
    unittest.main()
