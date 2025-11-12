"""Tests for serialize utilities"""

import json
import re
from datetime import datetime, timezone

import pytest

from nucleoid.lib.serialize import serialize


class TestSerializeLib:
    """Test suite for serialize lib"""

    def test_serializes_an_array(self):
        """Test that serialize() correctly serializes a list"""
        input_value = [1, 2, 3]
        output = serialize(input_value, "source")
        assert json.loads(output) == [1, 2, 3]

    def test_serializes_a_map(self):
        """Test that serialize() correctly serializes a dict as a Map"""
        # In Python, we'll use dict but format it as a Map-like structure
        # For this test, we need to treat it as a plain object
        input_value = {"key": "value"}
        output = serialize(input_value, "source")
        assert output == '{key:"value"}'

    def test_serializes_a_set(self):
        """Test that serialize() correctly serializes a set"""
        input_value = {1, 2, 3}
        output = serialize(input_value, "source")
        # Sets are unordered, so we need to check all possible orderings
        # or we can check that it starts with "new Set([" and contains all elements
        assert output.startswith("new Set([")
        assert output.endswith("])")
        assert "1" in output
        assert "2" in output
        assert "3" in output

    def test_serializes_a_date(self):
        """Test that serialize() correctly serializes a datetime"""
        # Create a datetime equivalent to "2023-05-17T00:00:00Z"
        input_value = datetime(2023, 5, 17, 0, 0, 0, tzinfo=timezone.utc)
        output = serialize(input_value, "source")
        timestamp = int(input_value.timestamp() * 1000)
        assert output == f"new Date({timestamp})"

    def test_serializes_a_string(self):
        """Test that serialize() correctly serializes a string"""
        input_value = "hello"
        output = serialize(input_value, "source")
        assert output == '"hello"'

    def test_serializes_a_number(self):
        """Test that serialize() correctly serializes a number"""
        input_value = 123
        output = serialize(input_value, "source")
        assert output == "123"

    def test_serializes_a_boolean(self):
        """Test that serialize() correctly serializes a boolean"""
        input_value = True
        output = serialize(input_value, "source")
        assert output == "true"

    def test_serializes_a_function(self):
        """Test that serialize() correctly serializes a function"""
        def test_func():
            return "hello"

        output = serialize(test_func, "source")
        # Check that the output contains the function definition
        assert "def test_func" in output or "function test_func" in output
        assert "return" in output and "hello" in output

    def test_serializes_a_regexp(self):
        """Test that serialize() correctly serializes a regex pattern"""
        input_value = re.compile(r"hello", re.IGNORECASE)
        output = serialize(input_value, "source")
        # Python regex doesn't have the same string representation as JS
        # We just check that it contains the pattern
        assert "hello" in output

    def test_serializes_an_object(self):
        """Test that serialize() correctly serializes a dict"""
        input_value = {"key": "value"}
        output = serialize(input_value, "source")
        assert output == '{key:"value"}'

    def test_serializes_an_object_with_reference(self):
        """Test that serialize() handles circular references"""
        input_value = {"id": "id1", "prop": {"test": {}}}
        # Create circular reference
        input_value["prop"]["test"] = input_value

        output = serialize(input_value, "source")
        assert output == "{id:\"id1\",prop:{test:{$ref:{id:'id1',source:'source'}}}}"

    def test_serializes_none(self):
        """Test that serialize() correctly serializes None as null"""
        output = serialize(None, "source")
        assert output == "null"

    def test_serializes_float(self):
        """Test that serialize() correctly serializes floats"""
        input_value = 123.456
        output = serialize(input_value, "source")
        assert output == "123.456"

    def test_serializes_nested_objects(self):
        """Test that serialize() correctly serializes nested dictionaries"""
        input_value = {"outer": {"inner": "value"}}
        output = serialize(input_value, "source")
        assert output == '{outer:{inner:"value"}}'
