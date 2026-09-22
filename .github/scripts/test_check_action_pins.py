"""Regression tests for the third-party GitHub Action SHA policy in #727."""

from pathlib import Path
import subprocess
import tempfile
import unittest


CHECKER = Path(__file__).with_name("check_action_pins.rb")
WORKFLOW_PREFIX = "jobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n"


def run_checker(workflows=None, actions=None):
    """Run the real YAML policy gate against a disposable repository fixture."""
    with tempfile.TemporaryDirectory(prefix="elephc-action-pins-") as tmp:
        root = Path(tmp)
        for name, source in (workflows or {}).items():
            path = root / ".github" / "workflows" / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source)
        for name, source in (actions or {}).items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source)
        return subprocess.run(
            ["ruby", str(CHECKER), str(root)], capture_output=True, text=True, check=False,
        )


class ActionPinTests(unittest.TestCase):
    """Exercise action refs in block and flow YAML across workflow and action files."""

    def test_pinned_third_party_and_local_actions_pass(self):
        sha = "a" * 40
        source = (
            WORKFLOW_PREFIX
            + f"      - uses: dtolnay/rust-toolchain@{sha} # stable, reviewed\n"
            "      - uses: actions/checkout@v4\n"
            "      - uses: ./github/actions/local\n"
        )
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_tag_branch_short_sha_and_uppercase_sha_fail(self):
        refs = ["dtolnay/rust-toolchain@stable", "taiki-e/install-action@nextest",
                "owner/action@1234567", "owner/action@" + "A" * 40]
        for ref in refs:
            with self.subTest(ref=ref):
                result = run_checker(workflows={"ci.yml": WORKFLOW_PREFIX + f"      - uses: {ref}\n"})
                self.assertEqual(result.returncode, 1)
                self.assertIn("ci.yml:5", result.stderr)

    def test_lookalike_owner_and_quoted_refs_do_not_bypass_check(self):
        source = (WORKFLOW_PREFIX + "      - uses: 'actions-evil/checkout@v4'\n"
                  "      - uses: \"owner/action@v1\" # tag\n")
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr.count("third-party action must use"), 2)

    def test_flow_style_step_with_movable_ref_fails(self):
        source = "jobs: { check: { runs-on: ubuntu-latest, steps: [ { uses: owner/action@v1 } ] } }\n"
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 1)
        self.assertIn("ci.yml:1", result.stderr)
        self.assertIn("owner/action@v1", result.stderr)

    def test_flow_style_step_with_full_sha_passes(self):
        sha = "b" * 40
        source = f"jobs: {{ check: {{ runs-on: ubuntu-latest, steps: [ {{ uses: owner/action@{sha} }} ] }} }}\n"
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_script_text_is_not_mistaken_for_an_action_ref(self):
        source = WORKFLOW_PREFIX + "      - run: |\n          echo 'uses: owner/action@v1'\n"
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_nested_uses_configuration_does_not_count_as_an_action(self):
        source = ("jobs:\n  check:\n    runs-on: ubuntu-latest\n"
                  "    strategy: { matrix: { include: [ { uses: cache } ] } }\n"
                  "    steps:\n      - uses: actions/cache@v4\n"
                  "        with: { uses: cache }\n"
                  "        env: { uses: value }\n")
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_reusable_workflow_job_uses_must_be_pinned(self):
        source = "jobs: { reuse: { uses: owner/repo/.github/workflows/reuse.yml@main, with: { uses: cache } } }\n"
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 1)
        self.assertIn("ci.yml:1", result.stderr)
        self.assertEqual(result.stderr.count("third-party action must use"), 1)

    def test_duplicate_jobs_sections_cannot_hide_a_movable_ref(self):
        source = (WORKFLOW_PREFIX + "      - uses: actions/checkout@v4\n"
                  "jobs:\n  other: { uses: owner/repo/.github/workflows/reuse.yml@main }\n")
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 1)
        self.assertIn("owner/repo/.github/workflows/reuse.yml@main", result.stderr)

    def test_job_alias_cannot_hide_a_movable_ref(self):
        source = ("template: &jobs\n  check: { uses: owner/repo/.github/workflows/reuse.yml@main }\n"
                  "jobs: *jobs\n")
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 1)
        self.assertIn("aliases cannot stand in for workflow jobs", result.stderr)

    def test_repository_scans_composite_actions_inside_and_outside_github(self):
        result = run_checker(
            workflows={"ci.yml": WORKFLOW_PREFIX + "      - uses: owner/workflow-action@v1\n"},
            actions={
                ".github/actions/custom/action.yaml":
                    "runs:\n  using: composite\n  steps:\n    - uses: owner/composite-action@main\n",
                "tools/build-action/action.yml":
                    "runs: { using: composite, steps: [ { uses: owner/other-action@stable } ] }\n",
            },
        )
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr.count("third-party action must use"), 3)
        self.assertIn("ci.yml:5", result.stderr)
        self.assertIn(".github/actions/custom/action.yaml:4", result.stderr)
        self.assertIn("tools/build-action/action.yml:1", result.stderr)

    def test_non_composite_manifest_does_not_invent_a_step(self):
        result = run_checker(actions={
            "tools/docker-action/action.yml":
                "runs:\n  using: docker\n  image: Dockerfile\nmetadata:\n  uses: descriptive text\n",
        })
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_invalid_yaml_fails_closed(self):
        result = run_checker(workflows={"ci.yml": "jobs: { check: { steps: [ { uses: owner/action@v1\n"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("invalid YAML", result.stderr)


if __name__ == "__main__":
    unittest.main()
