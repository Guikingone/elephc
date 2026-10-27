"""Regression checks for the PHPT adapter's failure accounting and byte preservation."""
import tempfile
from pathlib import Path
import unittest
from replay import parse_sections, settings, command
import sys


class AdapterTests(unittest.TestCase):
    """Keep unsupported cases visible and preserve diagnostic-sensitive test bytes."""

    def test_preserves_file_and_expectation_bytes(self):
        """Inline HTML, blank diagnostics and NUL bytes survive PHPT extraction."""
        sections = parse_sections(b'--TEST--\nx\n--FILE--\n<?php echo "x";?>\nHTML\n--EXPECT--\n\n\x00x\n')
        self.assertEqual(sections['FILE'], b'<?php echo "x";?>\nHTML\n')
        self.assertEqual(sections['EXPECT'], b'\n\x00x\n')

    def test_rejects_duplicate_sections(self):
        """A duplicate section cannot silently replace upstream source or expectations."""
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            parse_sections(b'--TEST--\nx\n--FILE--\na\n--FILE--\nb\n')

    def test_unknown_sections_remain_visible(self):
        """The parser exposes unsupported adapters to the replay's explicit blocker check."""
        self.assertIn('POST', parse_sections(b'--TEST--\nx\n--POST--\nx=1\n'))

    def test_merged_output_preserves_cross_descriptor_write_order(self):
        """PHPT scoring preserves stderr/stdout ordering and never hides duplicate diagnostics."""
        result = command([sys.executable, '-c',
                          'import os; os.write(2,b"error\\n"); os.write(1,b"display\\n"); os.write(2,b"tail\\n")'],
                         None, None, 5, merge_output=True)
        self.assertEqual(result['stdout'], b'error\ndisplay\ntail\n')
        self.assertEqual(result['stderr'], b'')
        self.assertEqual(result['exit'], 0)

    def test_ini_and_env_expand_in_order(self):
        """Later PHPT directives override defaults and use the sandbox's fixture path."""
        with tempfile.TemporaryDirectory() as directory:
            ini, env = settings({'INI': b'display_errors=0\nopcache.file_cache="{PWD}/cache"\n',
                                 'ENV': b'PHPT_FIXTURE={PWD}\n'}, Path(directory))
            self.assertEqual(ini[-2:], ['display_errors=0', 'opcache.file_cache=' + directory + '/cache'])
            self.assertEqual(env['PHPT_FIXTURE'], directory)

    def test_opcode_dump_phpts_are_classified_out_of_scope(self):
        """A non-zero opcache.opt_debug_level asks php for Zend opcode dumps; zero does not."""
        from replay import dumps_zend_opcodes
        self.assertTrue(dumps_zend_opcodes({'INI': b'opcache.enable=1\nopcache.opt_debug_level=0x20000\n'}))
        self.assertTrue(dumps_zend_opcodes({'INI': b'opcache.opt_debug_level = 4\n'}))
        self.assertFalse(dumps_zend_opcodes({'INI': b'opcache.opt_debug_level=0\n'}))
        self.assertFalse(dumps_zend_opcodes({'INI': b'opcache.enable=1\n'}))
        self.assertFalse(dumps_zend_opcodes({}))


if __name__ == '__main__':
    unittest.main()
