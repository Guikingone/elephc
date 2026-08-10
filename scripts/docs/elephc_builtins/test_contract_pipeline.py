"""Focused tests for shared-contract builtin documentation generation."""

from __future__ import annotations

import sys
import unittest
from collections import Counter
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(HERE))

import extract  # noqa: E402
import render  # noqa: E402


class ContractPipelineTests(unittest.TestCase):
    """Exercise exceptional support routes and presentation validation."""

    @classmethod
    def setUpClass(cls) -> None:
        """Load the prebuilt exporter once for all contract assertions."""
        cls.records = extract.run_gen_builtins(REPO)
        cls.by_name = {record["name"]: record for record in cls.records}

    def test_all_non_registry_contract_routes_are_exported(self) -> None:
        """Keep the six constructs, four preludes, and four eval-only routes explicit."""
        routes = Counter(
            (record.get("aot") or {}).get("kind")
            for record in self.records
            if (record.get("aot") or {}).get("kind") != "registry"
        )
        self.assertEqual(
            routes,
            Counter(
                {
                    "language-construct": 5,
                    "dedicated-syntax": 1,
                    "prelude": 4,
                    "none": 4,
                }
            ),
        )

    def test_hash_init_and_exit_use_backend_contract_signatures(self) -> None:
        """Pin the prelude subset and construct default that previously drifted."""
        hash_init = self.by_name["hash_init"]
        self.assertTrue(hash_init["aot"]["supported"])
        self.assertEqual(hash_init["aot"]["kind"], "prelude")
        self.assertEqual(hash_init["aot"]["signature_override_reason"], "prelude-signature-subset")
        self.assertEqual([param["name"] for param in hash_init["aot"]["params"]], ["algo"])
        self.assertEqual(self.by_name["exit"]["params"][0]["default"], 0)

    def test_unknown_presentation_override_is_rejected(self) -> None:
        """Prevent dormant override keys from accumulating silently again."""
        with patch.dict(extract.AREA_BY_NAME, {"__unknown_contract": ("Misc", "Misc")}):
            with self.assertRaisesRegex(ValueError, "unknown contracts"):
                extract.validate_presentation_overrides(REPO, self.records)

    def test_prelude_availability_renders_both_effective_signatures(self) -> None:
        """Show the narrower AOT call and broader eval call without marking eval-only."""
        rendered = render._availability_section(
            {
                "name": "hash_init",
                "aot": self.by_name["hash_init"]["aot"],
                "eval": self.by_name["hash_init"]["eval"],
                "eval_only": False,
                "is_extension": False,
            }
        )
        self.assertIn("compiler-injected hash prelude", rendered)
        self.assertIn('hash_init(string $algo, int $flags = 0, string $key = "")', rendered)
        self.assertNotIn("Compiled (AOT)**: not available", rendered)


if __name__ == "__main__":
    unittest.main()
