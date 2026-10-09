"""Regression tests for builtin documentation metadata extraction."""

from pathlib import Path
import unittest

from scripts.docs.elephc_builtins.extract import (
    _builtin_to_dict,
    build_registry,
    find_prelude_declaration,
)


class BuiltinDocumentationExtractionTests(unittest.TestCase):
    """Pins metadata that must survive from the Rust registry to rendered docs."""

    def test_get_object_vars_examples_survive_registry_serialization(self) -> None:
        """The static builtin example must reach the JSON consumed by the renderer."""
        repo = Path(__file__).resolve().parents[3]
        builtin = next(
            item
            for item in build_registry(repo)
            if item.canonical_name == "get_object_vars"
        )

        exported = _builtin_to_dict(builtin)

        self.assertTrue(exported["examples"])
        self.assertIn("examples/get-object-vars/main.php", exported["examples"][0])

    def test_namespaced_php_prelude_declaration_is_located(self) -> None:
        """A canonical namespace is validated before matching its bare function."""
        source = """<?php
namespace Elephc\\Async {
function run(\\Closure $body): mixed {}
}
"""

        match = find_prelude_declaration(source, "elephc\\async\\run")

        self.assertIsNotNone(match)
        self.assertEqual("function run(", match.group(0))
        self.assertIsNone(find_prelude_declaration(source, "Other\\Async\\run"))


if __name__ == "__main__":
    unittest.main()
