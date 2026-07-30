#!/usr/bin/env python3

import hashlib
import json
import shlex
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "build-pinned-php-src.sh"
INVENTORY = REPO_ROOT / "docs" / "specs" / "wasm-inventory.json"


def run(*arguments: str, check: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(SCRIPT), *arguments],
        cwd=REPO_ROOT,
        check=check,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def git(repository: Path, *arguments: str) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repository), *arguments],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return completed.stdout.strip()


class PinnedPhpSrcBuildTests(unittest.TestCase):
    def test_repository_inventory_yields_the_four_canonical_pins(self) -> None:
        completed = run("--verify-pins-only", check=True)
        records = [line.split("\t") for line in completed.stdout.splitlines()]

        self.assertEqual([record[0] for record in records], ["8.2", "8.3", "8.4", "8.5"])
        for profile, tag, tag_object, tag_commit in records:
            self.assertRegex(tag, rf"^php-{profile}\.[0-9]+$")
            self.assertRegex(tag_object, r"^[0-9a-f]{40}$")
            self.assertRegex(tag_commit, r"^[0-9a-f]{40}$")
            self.assertNotEqual(tag_object, tag_commit)

    def test_missing_profile_is_rejected_before_fetch(self) -> None:
        document = json.loads(INVENTORY.read_text(encoding="utf-8"))
        document["metadata"]["pins"]["php_src"] = [
            pin
            for pin in document["metadata"]["pins"]["php_src"]
            if pin["profile"] != "8.5"
        ]
        with tempfile.TemporaryDirectory(prefix="elephc-pin-test-") as directory:
            inventory = Path(directory) / "inventory.json"
            inventory.write_text(json.dumps(document), encoding="utf-8")
            completed = run(
                "--inventory",
                str(inventory),
                "--verify-pins-only",
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("expected exactly php-src profiles", completed.stderr)

    def test_legacy_inventory_schema_is_rejected_before_fetch(self) -> None:
        document = json.loads(INVENTORY.read_text(encoding="utf-8"))
        document["metadata"]["schema"] = "elephc.wasm-inventory.v3"
        with tempfile.TemporaryDirectory(prefix="elephc-pin-test-") as directory:
            inventory = Path(directory) / "inventory.json"
            inventory.write_text(json.dumps(document), encoding="utf-8")
            completed = run(
                "--inventory",
                str(inventory),
                "--verify-pins-only",
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("metadata.schema must be 'elephc.wasm-inventory.v4'", completed.stderr)

    def test_malformed_commit_is_rejected_before_fetch(self) -> None:
        document = json.loads(INVENTORY.read_text(encoding="utf-8"))
        document["metadata"]["pins"]["php_src"][0]["tag_commit"] = "not-a-commit"
        with tempfile.TemporaryDirectory(prefix="elephc-pin-test-") as directory:
            inventory = Path(directory) / "inventory.json"
            inventory.write_text(json.dumps(document), encoding="utf-8")
            completed = run(
                "--inventory",
                str(inventory),
                "--verify-pins-only",
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("tag_commit must be 40 lowercase hex characters", completed.stderr)

    def test_checkout_verification_rejects_tag_object_mismatch(self) -> None:
        with tempfile.TemporaryDirectory(prefix="elephc-provenance-test-") as directory:
            repository = Path(directory) / "repo"
            repository.mkdir()
            git(repository, "init", "--quiet")
            git(repository, "config", "user.name", "Elephc Test")
            git(repository, "config", "user.email", "test@elephc.invalid")
            (repository / "source.txt").write_text("first\n", encoding="utf-8")
            git(repository, "add", "source.txt")
            git(repository, "commit", "--quiet", "-m", "first")
            first = git(repository, "rev-parse", "HEAD")
            git(repository, "tag", "php-8.2.99")
            (repository / "source.txt").write_text("second\n", encoding="utf-8")
            git(repository, "commit", "--quiet", "-am", "second")
            second = git(repository, "rev-parse", "HEAD")
            git(repository, "checkout", "--quiet", "--detach", second)

            completed = self.run_verify_checkout(
                repository, "php-8.2.99", second, second
            )

        self.assertNotEqual(first, second)
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn(
            f"tag php-8.2.99 object is {first}, expected inventory object {second}",
            completed.stderr,
        )

    def test_checkout_verification_rejects_peeled_commit_mismatch(self) -> None:
        with tempfile.TemporaryDirectory(prefix="elephc-provenance-test-") as directory:
            repository = Path(directory) / "repo"
            repository.mkdir()
            git(repository, "init", "--quiet")
            git(repository, "config", "user.name", "Elephc Test")
            git(repository, "config", "user.email", "test@elephc.invalid")
            (repository / "source.txt").write_text("source\n", encoding="utf-8")
            git(repository, "add", "source.txt")
            git(repository, "commit", "--quiet", "-m", "source")
            commit = git(repository, "rev-parse", "HEAD")
            git(repository, "tag", "-a", "php-8.2.99", "-m", "annotated")
            tag_object = git(repository, "rev-parse", "refs/tags/php-8.2.99")
            git(repository, "checkout", "--quiet", "--detach", commit)
            wrong_commit = "0" * 40

            completed = self.run_verify_checkout(
                repository, "php-8.2.99", tag_object, wrong_commit
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn(
            f"tag php-8.2.99 peels to {commit}, expected inventory commit {wrong_commit}",
            completed.stderr,
        )

    def test_checkout_verification_rejects_dirty_tree(self) -> None:
        with tempfile.TemporaryDirectory(prefix="elephc-provenance-test-") as directory:
            repository = Path(directory) / "repo"
            repository.mkdir()
            git(repository, "init", "--quiet")
            git(repository, "config", "user.name", "Elephc Test")
            git(repository, "config", "user.email", "test@elephc.invalid")
            (repository / "source.txt").write_text("clean\n", encoding="utf-8")
            git(repository, "add", "source.txt")
            git(repository, "commit", "--quiet", "-m", "source")
            commit = git(repository, "rev-parse", "HEAD")
            git(repository, "tag", "php-8.2.99")
            git(repository, "checkout", "--quiet", "--detach", commit)
            (repository / "untracked.txt").write_text("dirty\n", encoding="utf-8")

            completed = self.run_verify_checkout(
                repository, "php-8.2.99", commit, commit
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("checkout is not clean", completed.stderr)

    def test_annotated_tag_object_and_peeled_commit_are_distinguished(self) -> None:
        with tempfile.TemporaryDirectory(prefix="elephc-provenance-test-") as directory:
            repository = Path(directory) / "repo"
            repository.mkdir()
            git(repository, "init", "--quiet")
            git(repository, "config", "user.name", "Elephc Test")
            git(repository, "config", "user.email", "test@elephc.invalid")
            (repository / "source.txt").write_text("source\n", encoding="utf-8")
            git(repository, "add", "source.txt")
            git(repository, "commit", "--quiet", "-m", "source")
            commit = git(repository, "rev-parse", "HEAD")
            git(repository, "tag", "-a", "php-8.2.99", "-m", "annotated")
            tag_object = git(repository, "rev-parse", "refs/tags/php-8.2.99")
            git(repository, "checkout", "--quiet", "--detach", commit)

            completed = self.run_verify_checkout(
                repository, "php-8.2.99", tag_object, commit
            )

        self.assertNotEqual(tag_object, commit)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertEqual(
            completed.stdout.strip().split("\t"),
            [tag_object, commit, commit],
        )

    def test_hash_verification_rejects_tampered_provenance(self) -> None:
        with tempfile.TemporaryDirectory(prefix="elephc-provenance-test-") as directory:
            root = Path(directory)
            provenance = root / "provenance.json"
            hashes = root / "hashes.sha256"
            provenance.write_text('{"commit":"first"}\n', encoding="utf-8")
            digest = hashlib.sha256(provenance.read_bytes()).hexdigest()
            hashes.write_text(f"{digest}  provenance.json\n", encoding="utf-8")
            provenance.write_text('{"commit":"tampered"}\n', encoding="utf-8")
            command = (
                f"source {shlex.quote(str(SCRIPT))}; "
                'verify_hash_manifest "$1" "$2"'
            )
            completed = subprocess.run(
                ["bash", "-c", command, "verify-hashes", str(root), str(hashes)],
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("SHA-256 mismatch", completed.stderr)

    def test_profile_provenance_records_both_git_objects(self) -> None:
        tag_object = "1" * 40
        tag_commit = "2" * 40
        digest = "a" * 64
        with tempfile.TemporaryDirectory(prefix="elephc-provenance-test-") as directory:
            destination = Path(directory) / "provenance.json"
            command = (
                f"source {shlex.quote(str(SCRIPT))}; "
                'write_profile_provenance "$1" 8.2 php-8.2.99 "$2" "$3" '
                '"$2" "$3" "$3" "$4" 8.2.99 123 "$4" "$4" '
                '"git test" "autoconf test" "bison test" "re2c test" '
                '"make test" "cc test" --prefix=/install --disable-all'
            )
            completed = subprocess.run(
                [
                    "bash",
                    "-c",
                    command,
                    "write-provenance",
                    str(destination),
                    tag_object,
                    tag_commit,
                    digest,
                ],
                cwd=REPO_ROOT,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            document = json.loads(destination.read_text(encoding="utf-8"))

        self.assertEqual(document["source"]["inventory_tag_object"], tag_object)
        self.assertEqual(document["source"]["inventory_tag_commit"], tag_commit)
        self.assertEqual(document["source"]["tag_object"], tag_object)
        self.assertEqual(document["source"]["tag_commit"], tag_commit)
        self.assertEqual(document["source"]["head"], tag_commit)
        self.assertEqual(
            document["build"]["configure_args"],
            ["--prefix=/install", "--disable-all"],
        )

    @staticmethod
    def run_verify_checkout(
        repository: Path, tag: str, tag_object: str, tag_commit: str
    ) -> subprocess.CompletedProcess[str]:
        command = (
            f"source {shlex.quote(str(SCRIPT))}; "
            'verify_checkout "$1" "$2" "$3" "$4"'
        )
        return subprocess.run(
            [
                "bash",
                "-c",
                command,
                "verify-checkout",
                str(repository),
                tag,
                tag_object,
                tag_commit,
            ],
            cwd=REPO_ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )


if __name__ == "__main__":
    unittest.main()
