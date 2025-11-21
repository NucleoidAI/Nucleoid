"""Tests for deep comparison and merge utilities"""

import pytest

from nucleoid.lib.deep import deep_equal, deep_merge, equal


class TestDeepLib:
    """Test suite for deep lib"""

    def test_compares_two_objects(self):
        """Test that deep_equal() correctly compares nested objects"""
        obj1 = {"a": 1, "b": 2, "c": {"d": 3, "e": 4}}
        obj2 = {"a": 1, "b": 2, "c": {"d": 3, "e": 4}}
        obj3 = {"a": 1, "b": 2, "c": {"d": 3, "e": 5}}

        assert deep_equal(obj1, obj2) is True
        assert deep_equal(obj1, obj3) is False

    def test_merges_two_objects(self):
        """Test that deep_merge() correctly merges nested objects"""
        obj1 = {"a": 1, "b": 2, "c": {"d": 3, "e": 4}}
        obj2 = {"b": 3, "c": {"d": 4, "f": 5}, "g": 6}
        obj3 = {"a": 1, "b": 3, "c": {"d": 4, "e": 4, "f": 5}, "g": 6}

        result = deep_merge(obj1, obj2)
        assert result == obj3

    def test_deep_equal_with_same_reference(self):
        """Test that deep_equal returns True for same object reference"""
        obj = {"a": 1, "b": 2}
        assert deep_equal(obj, obj) is True

    def test_deep_equal_with_none(self):
        """Test that deep_equal handles None values"""
        assert deep_equal(None, None) is True
        assert deep_equal(None, {"a": 1}) is False
        assert deep_equal({"a": 1}, None) is False

    def test_deep_equal_with_lists(self):
        """Test that deep_equal correctly compares lists"""
        list1 = [1, 2, [3, 4]]
        list2 = [1, 2, [3, 4]]
        list3 = [1, 2, [3, 5]]

        assert deep_equal(list1, list2) is True
        assert deep_equal(list1, list3) is False

    def test_deep_equal_with_different_types(self):
        """Test that deep_equal returns False for different types"""
        assert deep_equal({"a": 1}, [1, 2]) is False
        assert deep_equal(123, "123") is False

    def test_deep_equal_with_primitives(self):
        """Test that deep_equal works with primitive values"""
        assert deep_equal(42, 42) is True
        assert deep_equal(42, 43) is False
        assert deep_equal("hello", "hello") is True
        assert deep_equal("hello", "world") is False

    def test_equal_function(self):
        """Test that equal() performs simple equality comparison"""
        assert equal(42, 42) is True
        assert equal(42, 43) is False
        assert equal("hello", "hello") is True
        assert equal([1, 2], [1, 2]) is True

    def test_deep_merge_preserves_target(self):
        """Test that deep_merge doesn't modify the original target"""
        obj1 = {"a": 1, "b": 2}
        obj2 = {"b": 3, "c": 4}
        result = deep_merge(obj1, obj2)

        # Original should be unchanged
        assert obj1 == {"a": 1, "b": 2}
        # Result should have merged values
        assert result == {"a": 1, "b": 3, "c": 4}

    def test_deep_merge_with_nested_objects(self):
        """Test that deep_merge correctly handles deeply nested objects"""
        obj1 = {"a": {"b": {"c": 1}}}
        obj2 = {"a": {"b": {"d": 2}}}
        result = deep_merge(obj1, obj2)

        assert result == {"a": {"b": {"c": 1, "d": 2}}}

    def test_deep_merge_overwrites_non_object_values(self):
        """Test that deep_merge overwrites non-object values"""
        obj1 = {"a": 1, "b": "hello"}
        obj2 = {"a": 2, "b": "world"}
        result = deep_merge(obj1, obj2)

        assert result == {"a": 2, "b": "world"}
