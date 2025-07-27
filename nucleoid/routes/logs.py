"""
Logs API routes for Nucleoid runtime.

This module provides endpoints for accessing runtime logs and datastore tail information.
"""

from typing import Any

from fastapi import APIRouter

from .. import datastore

router = APIRouter()


@router.get("/logs")
async def get_logs() -> list[Any]:
    """
    Get the tail of the datastore logs.
    
    Returns the most recent log entries from the datastore, useful for
    debugging and monitoring runtime activity.
    
    Returns:
        List of recent log entries
    """
    return datastore.tail()
