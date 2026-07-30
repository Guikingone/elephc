#!/usr/bin/env python3
"""Validate and aggregate exact php-src/WASM oracle evidence."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Sequence

from wasm_oracle import (
    AggregateError,
    CaptureError,
    ContractError,
    OracleContract,
    aggregate_exact,
    aggregate_generated_suite,
    load_capture_record,
)


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Fail-closed php-src/WASM oracle contract and evidence tools."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=_repo_root(),
        help="Elephc checkout containing docs/specs/wasm-inventory.json.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser(
        "contract",
        help="Validate the pinned inventory/specification contract and print it.",
    )

    validate = subparsers.add_parser(
        "validate-record",
        help="Validate one capture record against the current pinned contract.",
    )
    validate.add_argument("record", type=Path)

    aggregate = subparsers.add_parser(
        "aggregate",
        help="Require and compare the exact fixture/profile/runtime product.",
    )
    aggregate.add_argument(
        "--fixture",
        action="append",
        required=True,
        help="Expected fixture id; repeat for every matrix fixture.",
    )
    aggregate.add_argument(
        "--record",
        action="append",
        required=True,
        type=Path,
        help="Path to one capture record; repeat for every matrix cell.",
    )
    aggregate.add_argument(
        "--output",
        type=Path,
        help="Create this aggregate JSON file; omit to write to stdout.",
    )

    aggregate_suite = subparsers.add_parser(
        "aggregate-suite",
        help="Validate frames and aggregate the exact committed fixture suite.",
    )
    aggregate_suite.add_argument(
        "--record",
        action="append",
        required=True,
        type=Path,
        help="Path to one capture record; repeat for all 64 suite cells.",
    )
    aggregate_suite.add_argument(
        "--output",
        required=True,
        type=Path,
        help="Create this aggregate JSON file without overwriting it.",
    )
    return parser


def _json_text(value: Any) -> str:
    return json.dumps(
        value,
        ensure_ascii=True,
        indent=2,
        sort_keys=True,
        separators=(",", ": "),
    ) + "\n"


def _write_output(path: Path | None, value: Any) -> None:
    text = _json_text(value)
    if path is None:
        sys.stdout.write(text)
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open("x", encoding="utf-8", newline="\n") as output:
            output.write(text)
    except FileExistsError as error:
        raise AggregateError(
            f"refusing to overwrite existing evidence file: {path}"
        ) from error


def main(argv: Sequence[str] | None = None) -> int:
    """Run one strict contract, record-validation, or aggregation operation."""

    arguments = _parser().parse_args(argv)
    try:
        contract = OracleContract.load(arguments.repo_root)
        if arguments.command == "contract":
            _write_output(None, contract.to_dict())
        elif arguments.command == "validate-record":
            record = load_capture_record(arguments.record, contract)
            _write_output(None, record.to_dict())
        elif arguments.command == "aggregate":
            records = [
                load_capture_record(path, contract) for path in arguments.record
            ]
            result = aggregate_exact(contract, arguments.fixture, records)
            _write_output(arguments.output, result.to_dict())
        elif arguments.command == "aggregate-suite":
            records = [
                load_capture_record(path, contract) for path in arguments.record
            ]
            result = aggregate_generated_suite(contract, records)
            _write_output(arguments.output, result.to_dict())
        else:  # pragma: no cover - argparse guarantees a registered command
            raise AggregateError(f"unsupported command: {arguments.command}")
    except (ContractError, CaptureError, AggregateError) as error:
        print(f"wasm_php_oracle: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
