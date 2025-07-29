"""Tests for Nucleoid AI data store."""

import pytest
from datetime import datetime
from nucleoidai.datastore import DataStore
from nucleoidai.types import Data, Result, Event


class TestDataStore:
    """Test data store functionality."""
    
    def setup_method(self):
        """Set up test environment."""
        self.datastore = DataStore()
        self.datastore.init()
    
    def test_init_datastore(self):
        """Test datastore initialization."""
        assert self.datastore is not None
    
    def test_write_and_read(self):
        """Test writing and reading data."""
        test_data = Data(
            string="test statement",
            declarative=False,
            result=Result(nuc=[], value="test"),
            time=100.0,
            date=datetime.now(),
            error=False,
            events=[]
        )
        
        # Write data
        written = self.datastore.write(test_data)
        assert written is not None
        
        # Read data
        all_data = self.datastore.read()
        assert len(all_data) > 0
    
    def test_clear_datastore(self):
        """Test clearing datastore."""
        # Add some data first
        test_data = Data(
            string="test",
            result=Result(nuc=[], value="test"),
            time=50.0,
            date=datetime.now(),
            events=[]
        )
        self.datastore.write(test_data)
        
        # Clear and verify
        self.datastore.clear()
        all_data = self.datastore.read()
        assert len(all_data) == 0
    
    def test_tail_functionality(self):
        """Test tail functionality."""
        # Add multiple entries
        for i in range(5):
            test_data = Data(
                string=f"test {i}",
                result=Result(nuc=[], value=i),
                time=i * 10.0,
                date=datetime.now(),
                events=[]
            )
            self.datastore.write(test_data)
        
        # Get last 3 entries
        tail_data = self.datastore.tail(3)
        assert len(tail_data) <= 3