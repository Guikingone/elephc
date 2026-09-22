"""Regression tests for the third-party GitHub Action SHA policy in #727."""

from pathlib import Path
import subprocess
import tempfile
import unittest


CHECKER = Path(__file__).with_name("check_action_pins.rb")


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
            f"steps:\n  - uses: dtolnay/rust-toolchain@{sha} # stable, reviewed\n"
            "  - uses: actions/checkout@v4\n"
            "  - uses: ./github/actions/local\n"
        )
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_tag_branch_short_sha_and_uppercase_sha_fail(self):
        refs = ["dtolnay/rust-toolchain@stable", "taiki-e/install-action@nextest",
                "owner/action@1234567", "owner/action@" + "A" * 40]
        for ref in refs:
            with self.subTest(ref=ref):
                result = run_checker(workflows={"ci.yml": f"steps:\n  - uses: {ref}\n"})
                self.assertEqual(result.returncode, 1)
                self.assertIn("ci.yml:2", result.stderr)

    def test_lookalike_owner_and_quoted_refs_do_not_bypass_check(self):
        source = "steps:\n  - uses: 'actions-evil/checkout@v4'\n  - uses: \"owner/action@v1\" # tag\n"
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr.count("third-party action must use"), 2)

    def test_flow_style_step_with_movable_ref_fails(self):
        result = run_checker(workflows={"ci.yml": "steps: [ { uses: owner/action@v1 } ]\n"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("ci.yml:1", result.stderr)
        self.assertIn("owner/action@v1", result.stderr)

    def test_flow_style_step_with_full_sha_passes(self):
        sha = "b" * 40
        result = run_checker(workflows={"ci.yml": f"steps: [ {{ uses: owner/action@{sha} }} ]\n"})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_script_text_is_not_mistaken_for_an_action_ref(self):
        source = "steps:\n  - run: |\n      echo 'uses: owner/action@v1'\n"
        result = run_checker(workflows={"ci.yml": source})
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_repository_scans_composite_actions_inside_and_outside_github(self):
        result = run_checker(
            workflows={"ci.yml": "steps:\n  - uses: owner/workflow-action@v1\n"},
            actions={
                ".github/actions/custom/action.yaml":
                    "runs:\n  using: composite\n  steps:\n    - uses: owner/composite-action@main\n",
                "tools/build-action/action.yml":
                    "runs: { using: composite, steps: [ { uses: owner/other-action@stable } ] }\n",
            },
        )
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr.count("third-party action must use"), 3)
        self.assertIn("ci.yml:2", result.stderr)
        self.assertIn(".github/actions/custom/action.yaml:4", result.stderr)
        self.assertIn("tools/build-action/action.yml:1", result.stderr)

    def test_non_composite_manifest_does_not_invent_a_step(self):
        result = run_checker(actions={
            "tools/docker-action/action.yml":
                "runs:\n  using: docker\n  image: Dockerfile\nmetadata:\n  uses: descriptive text\n",
        })
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_invalid_yaml_fails_closed(self):
        result = run_checker(workflows={"ci.yml": "steps: [ { uses: owner/action@v1\n"})
        self.assertEqual(result.returncode, 1)
        self.assertIn("invalid YAML", result.stderr)


if __name__ == "__main__":
    unittest.main()
