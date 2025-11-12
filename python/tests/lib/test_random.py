"""Tests for random string generation utilities"""

import pytest
from nucleoid.lib.random import random


class TestRandomLib:
    """Test suite for random lib"""

    def test_returns_random_string_with_default_length(self):
        """Test that random() returns a string with default length of 16"""
        result = random()
        assert len(result) == 16

    def test_returns_random_string_with_given_length(self):
        """Test that random() returns a string with the specified length"""
        result = random(32)
        assert len(result) == 32

    def test_random_string_starts_with_letter(self):
        """Test that random string always starts with a letter"""
        result = random()
        assert result[0].isalpha()

    def test_alpha_only_contains_letters(self):
        """Test that alphanumeric=False produces only letters"""
        result = random(100, alphanumeric=False)
        assert result.isalpha()

    def test_alphanumeric_can_contain_digits(self):
        """Test that alphanumeric=True allows digits (though not guaranteed)"""
        # Generate multiple strings to increase chance of getting digits
        # This test just verifies the function runs with alphanumeric=True
        for _ in range(10):
            result = random(50, alphanumeric=True)
            assert len(result) == 50
            assert result[0].isalpha()  # First char is always alpha
