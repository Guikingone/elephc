#!/usr/bin/env python3
"""Build a complete PR/direct-commit ledger for an Elephc release range."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

FIELD_SEPARATOR = "\x1f"
RECORD_SEPARATOR = "\x1e"
MERGE_PR_RE = re.compile(r"^Merge pull request #(\d+)\b")
SQUASH_PR_RE = re.compile(r"\(#(\d+)\)$")


class CommandFailure(RuntimeError):
    """Describe a failed external command without discarding its diagnostics."""

    def __init__(self, command: list[str], result: subprocess.CompletedProcess[str]):
        rendered = " ".join(command)
        detail = result.stderr.strip() or result.stdout.strip() or "no output"
        super().__init__(f"{rendered}: {detail}")


def run(
    command: list[str], cwd: Path, *, check: bool = True
) -> subprocess.CompletedProcess[str]:
    """Run a command in the repository and optionally raise on failure."""

    result = subprocess.run(
        command, cwd=cwd, text=True, capture_output=True, check=False
    )
    if check and result.returncode != 0:
        raise CommandFailure(command, result)
    return result


def git(
    repo: Path, *arguments: str, check: bool = True
) -> subprocess.CompletedProcess[str]:
    """Run Git with the supplied arguments in the selected repository."""

    return run(["git", *arguments], repo, check=check)


def resolve_commit(repo: Path, reference: str) -> str:
    """Resolve a ref or tag to its underlying commit SHA."""

    return git(repo, "rev-parse", f"{reference}^{{commit}}").stdout.strip()


def parse_log(
    repo: Path, revision_range: str, *, first_parent: bool
) -> list[dict[str, Any]]:
    """Return structured commits for a revision range in chronological order."""

    command = ["log", "--reverse"]
    if first_parent:
        command.append("--first-parent")
    command.extend(
        [
            f"--format=%H{FIELD_SEPARATOR}%P{FIELD_SEPARATOR}%cI{FIELD_SEPARATOR}%an{FIELD_SEPARATOR}%s{RECORD_SEPARATOR}",
            revision_range,
        ]
    )
    output = git(repo, *command).stdout
    commits: list[dict[str, Any]] = []
    for raw_record in output.split(RECORD_SEPARATOR):
        record = raw_record.strip("\n")
        if not record:
            continue
        fields = record.split(FIELD_SEPARATOR, 4)
        if len(fields) != 5:
            raise RuntimeError(f"unexpected git log record: {record!r}")
        sha, parents, committed_at, author, subject = fields
        commits.append(
            {
                "sha": sha,
                "parents": parents.split(),
                "committed_at": committed_at,
                "author": author,
                "subject": subject,
            }
        )
    return commits


def parse_pr_number(subject: str) -> int | None:
    """Extract a PR number from GitHub merge or squash commit subjects."""

    match = MERGE_PR_RE.search(subject) or SQUASH_PR_RE.search(subject)
    return int(match.group(1)) if match else None


def infer_repository(repo: Path) -> str:
    """Ask GitHub CLI for the current repository's owner/name identity."""

    result = run(["gh", "repo", "view", "--json", "nameWithOwner"], repo)
    payload = json.loads(result.stdout)
    return str(payload["nameWithOwner"])


def associated_pr_numbers(
    repo: Path, repository: str, sha: str, base_branch: str
) -> list[int]:
    """Return merged PRs targeting the release branch that contain a commit."""

    result = run(
        [
            "gh",
            "api",
            f"repos/{repository}/commits/{sha}/pulls",
            "--header",
            "Accept: application/vnd.github+json",
        ],
        repo,
    )
    payload = json.loads(result.stdout)
    return sorted(
        {
            int(item["number"])
            for item in payload
            if item.get("merged_at") and item.get("base", {}).get("ref") == base_branch
        }
    )


def pull_request_metadata(repo: Path, repository: str, number: int) -> dict[str, Any]:
    """Load the GitHub fields needed to verify and inspect a merged PR."""

    result = run(["gh", "api", f"repos/{repository}/pulls/{number}"], repo)
    payload = json.loads(result.stdout)
    return {
        "number": int(payload["number"]),
        "title": payload.get("title", ""),
        "url": payload.get("html_url", ""),
        "merged_at": payload.get("merged_at"),
        "merge_commit_sha": payload.get("merge_commit_sha"),
        "base_branch": payload.get("base", {}).get("ref"),
        "head_sha": payload.get("head", {}).get("sha"),
        "author": payload.get("user", {}).get("login"),
        "labels": [label.get("name", "") for label in payload.get("labels", [])],
        "changed_files": payload.get("changed_files"),
        "additions": payload.get("additions"),
        "deletions": payload.get("deletions"),
    }


def introduced_commits(repo: Path, commit: dict[str, Any]) -> list[str]:
    """Expand commits introduced by one first-parent integration point."""

    parents = commit["parents"]
    introduced = {commit["sha"]}
    if len(parents) > 1:
        first_parent = parents[0]
        for merged_parent in parents[1:]:
            output = git(
                repo, "rev-list", "--reverse", f"{first_parent}..{merged_parent}"
            ).stdout
            introduced.update(line for line in output.splitlines() if line)
    return sorted(introduced)


def is_ancestor(repo: Path, ancestor: str, descendant: str) -> bool:
    """Return whether one commit is reachable from another."""

    result = git(repo, "merge-base", "--is-ancestor", ancestor, descendant, check=False)
    if result.returncode not in (0, 1):
        raise CommandFailure(
            ["git", "merge-base", "--is-ancestor", ancestor, descendant], result
        )
    return result.returncode == 0


def markdown_report(inventory: dict[str, Any]) -> str:
    """Render the inventory as a compact human-readable audit table."""

    lines = [
        "# Release inventory",
        "",
        f"- Repository: `{inventory['repository']}`",
        (
            f"- Range: `{inventory['from_tag']}` (`{inventory['from_sha']}`) .. "
            f"`{inventory['head_ref']}` (`{inventory['head_sha']}`)"
        ),
        f"- Coverage: {inventory['accounted_commit_count']}/{inventory['total_commit_count']} commits",
        f"- Pull requests: {inventory['pull_request_count']}",
        f"- Direct commits: {inventory['direct_commit_count']}",
        "",
        "| Kind | Main commit | PR | Date | Subject | Introduced commits |",
        "|---|---|---:|---|---|---:|",
    ]
    for item in inventory["integrations"]:
        pr = f"#{item['pr_number']}" if item.get("pr_number") else "-"
        subject = str(item["subject"]).replace("|", "\\|")
        lines.append(
            f"| {item['kind']} | `{item['sha'][:12]}` | {pr} | "
            f"{item['committed_at'][:10]} | {subject} | {len(item['introduced_commits'])} |"
        )
    if inventory["unaccounted_commits"]:
        lines.extend(
            [
                "",
                "Unaccounted commits:",
                *[f"- `{sha}`" for sha in inventory["unaccounted_commits"]],
            ]
        )
    if inventory["errors"]:
        lines.extend(["", "Errors:", *[f"- {error}" for error in inventory["errors"]]])
    return "\n".join(lines) + "\n"


def build_inventory(args: argparse.Namespace) -> dict[str, Any]:
    """Construct and verify the complete release-range inventory."""

    repo = Path(args.repo_root).resolve()
    top_level = Path(git(repo, "rev-parse", "--show-toplevel").stdout.strip())
    from_sha = resolve_commit(top_level, args.from_tag)
    head_sha = resolve_commit(top_level, args.head)
    if not is_ancestor(top_level, from_sha, head_sha):
        raise RuntimeError(
            f"release tag {args.from_tag} is not an ancestor of {args.head}"
        )

    repository = args.repo
    github_enabled = not args.no_github
    if github_enabled:
        if shutil.which("gh") is None:
            raise RuntimeError("GitHub CLI is required unless --no-github is selected")
        repository = repository or infer_repository(top_level)
    repository = repository or "offline/unknown"

    revision_range = f"{from_sha}..{head_sha}"
    all_commits = parse_log(top_level, revision_range, first_parent=False)
    first_parent_commits = parse_log(top_level, revision_range, first_parent=True)
    all_commit_shas = {commit["sha"] for commit in all_commits}
    integrations: list[dict[str, Any]] = []
    errors: list[str] = []
    accounted: set[str] = set()
    pr_numbers: set[int] = set()

    for commit in first_parent_commits:
        pr_number = parse_pr_number(commit["subject"])
        association_verified = pr_number is not None
        association_lookup_failed = False
        if pr_number is None and github_enabled:
            try:
                associated = associated_pr_numbers(
                    top_level, repository, commit["sha"], args.base_branch
                )
                association_verified = True
                if len(associated) == 1:
                    pr_number = associated[0]
                elif len(associated) > 1:
                    errors.append(
                        f"{commit['sha']}: ambiguous associated PRs {associated}"
                    )
                    association_lookup_failed = True
            except (CommandFailure, json.JSONDecodeError) as error:
                errors.append(f"{commit['sha']}: PR association lookup failed: {error}")
                association_lookup_failed = True

        if pr_number is not None:
            kind = "pull_request"
            pr_numbers.add(pr_number)
        elif github_enabled:
            kind = "unresolved" if association_lookup_failed else "direct_commit"
        else:
            kind = "direct_unverified"

        expanded = introduced_commits(top_level, commit)
        accounted.update(expanded)
        integrations.append(
            {
                **commit,
                "kind": kind,
                "pr_number": pr_number,
                "association_verified": association_verified,
                "introduced_commits": expanded,
            }
        )

    pull_requests: dict[str, Any] = {}
    if github_enabled:
        for number in sorted(pr_numbers):
            try:
                metadata = pull_request_metadata(top_level, repository, number)
                pull_requests[str(number)] = metadata
                if not metadata["merged_at"]:
                    errors.append(f"PR #{number} is not merged")
                if metadata["base_branch"] != args.base_branch:
                    errors.append(
                        f"PR #{number} targets {metadata['base_branch']!r}, not {args.base_branch!r}"
                    )
                merge_sha = metadata["merge_commit_sha"]
                if merge_sha and not is_ancestor(top_level, merge_sha, head_sha):
                    errors.append(
                        f"PR #{number} merge commit {merge_sha} is not in the candidate head"
                    )
            except (CommandFailure, json.JSONDecodeError) as error:
                errors.append(f"PR #{number}: metadata lookup failed: {error}")

    unaccounted = sorted(all_commit_shas - accounted)
    inventory = {
        "repository": repository,
        "from_tag": args.from_tag,
        "from_sha": from_sha,
        "head_ref": args.head,
        "head_sha": head_sha,
        "base_branch": args.base_branch,
        "github_verified": github_enabled,
        "total_commit_count": len(all_commit_shas),
        "accounted_commit_count": len(all_commit_shas & accounted),
        "pull_request_count": len(pr_numbers),
        "direct_commit_count": sum(
            item["kind"] == "direct_commit" for item in integrations
        ),
        "integrations": integrations,
        "pull_requests": pull_requests,
        "unaccounted_commits": unaccounted,
        "errors": errors,
    }
    return inventory


def parse_args() -> argparse.Namespace:
    """Parse command-line options for the release inventory."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo-root",
        default=".",
        help="repository checkout (default: current directory)",
    )
    parser.add_argument("--from-tag", required=True, help="last published release tag")
    parser.add_argument(
        "--head", default="origin/main", help="exact candidate ref or SHA"
    )
    parser.add_argument(
        "--repo", help="GitHub owner/name; inferred through gh when omitted"
    )
    parser.add_argument("--base-branch", default="main", help="expected PR base branch")
    parser.add_argument("--format", choices=("json", "markdown"), default="markdown")
    parser.add_argument(
        "--no-github",
        action="store_true",
        help="skip GitHub verification for offline diagnostics",
    )
    return parser.parse_args()


def main() -> int:
    """Generate the inventory and fail when reconciliation is incomplete."""

    args = parse_args()
    try:
        inventory = build_inventory(args)
    except (CommandFailure, RuntimeError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    if args.format == "json":
        print(json.dumps(inventory, indent=2, sort_keys=True))
    else:
        print(markdown_report(inventory), end="")
    return 2 if inventory["errors"] or inventory["unaccounted_commits"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
