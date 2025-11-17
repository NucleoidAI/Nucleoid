"""
Tests for Nucleoid runtime.

This test suite corresponds to the TypeScript test file:
typescript/src/test/nucleoid.spec.ts
"""

import pytest
from nucleoid import nucleoid
from nucleoid.lib.test import clear


class TestNucleoid:
    """Test suite for Nucleoid runtime in declarative mode."""

    @classmethod
    def setup_class(cls):
        """Setup class - start nucleoid with test mode and declarative mode."""
        nucleoid.start({'test': True, 'options': {'declarative': True}})

    def setup_method(self):
        """Setup method - clear state before each test."""
        clear()

    def test_runs_statements_in_the_state(self):
        """Test that nucleoid runs statements and stores them in the state.

        Note: The original TypeScript test uses JavaScript syntax:
            nucleoid.run("var i = 1 ;")
            assert nucleoid.run("i == 1") is True

        For the Python implementation, we use Python syntax until
        a JavaScript parser is implemented.
        """
        # Run a variable declaration using Python syntax
        nucleoid.run("i = 1")

        # Check that the variable is stored and can be retrieved
        result = nucleoid.run("i == 1")
        assert result is True
