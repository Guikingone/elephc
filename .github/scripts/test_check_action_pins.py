"""Regression tests for the third-party GitHub Action SHA policy in #727."""

from pathlib import Path
import unittest

import check_action_pins as pins


class ActionPinTests(unittest.TestCase):
    """Exercise tagged, pinned, local, and malformed `uses` references."""

    def test_pinned_third_party_and_local_actions_pass(self):
        sha = "a" * 40
        source = (
            f"- uses: dtolnay/rust-toolchain@{sha} # stable, reviewed\n"
            "- uses: actions/checkout@v4\n"
            "- uses: ./github/actions/local\n"
        )
        self.assertEqual(pins.check_text(source, Path("workflow.yml")), [])

    def test_tag_branch_short_sha_and_uppercase_sha_fail(self):
        refs = ["dtolnay/rust-toolchain@stable", "taiki-e/install-action@nextest",
                "owner/action@1234567", "owner/action@" + "A" * 40]
        for ref in refs:
            with self.subTest(ref=ref):
                errors = pins.check_text(f"  - uses: {ref}\n", Path("workflow.yml"))
                self.assertEqual(len(errors), 1)
                self.assertIn("workflow.yml:1", errors[0])

    def test_lookalike_owner_and_quoted_refs_do_not_bypass_check(self):
        source = "uses: 'actions-evil/checkout@v4'\nuses: \"owner/action@v1\" # tag\n"
        errors = pins.check_text(source, Path("action.yml"))
        self.assertEqual(len(errors), 2)

    def test_repository_workflows_and_composite_actions_are_checked(self):
        from tempfile import TemporaryDirectory

        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            workflow = root / ".github/workflows/ci.yml"
            action = root / ".github/actions/custom/action.yaml"
            workflow.parent.mkdir(parents=True)
            action.parent.mkdir(parents=True)
            workflow.write_text("steps:\n  - uses: owner/workflow-action@v1\n")
            action.write_text("runs:\n  steps:\n    - uses: owner/composite-action@main\n")
            errors = pins.check_repository(root)
            self.assertEqual(len(errors), 2)
            self.assertTrue(any("ci.yml:2" in error for error in errors))
            self.assertTrue(any("action.yaml:3" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
