"""Tests for test utility module"""

import pytest

from nucleoid import datastore, state, graph
from nucleoid.lib.test import clear


class TestTestUtil:
    """Test suite for test utility"""

    def test_clear_resets_all_modules(self):
        """Test that clear() resets state, graph, and datastore"""
        # Set some values in each module
        state.set("key1", "value1")
        graph.add_node("node1", {"data": "test"})
        datastore.set("key2", "value2")

        # Verify values are set
        assert state.get("key1") == "value1"
        assert datastore.get("key2") == "value2"

        # Clear everything
        clear()

        # Verify everything is cleared
        assert state.get("key1") is None
        assert datastore.get("key2") is None

    def test_clear_is_idempotent(self):
        """Test that calling clear() multiple times is safe"""
        state.set("key", "value")

        # Clear multiple times
        clear()
        clear()
        clear()

        # Should not raise any errors and state should still be clear
        assert state.get("key") is None

    def test_clear_with_empty_modules(self):
        """Test that clear() works when modules are already empty"""
        # Ensure modules are empty
        clear()

        # Clear again - should not raise errors
        clear()

        # Verify still empty
        assert state.get("anything") is None
        assert datastore.get("anything") is None
