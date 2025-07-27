"""
Metrics API routes for Nucleoid runtime.

This module provides endpoints for system metrics including memory usage
and other performance indicators.
"""


import psutil
from fastapi import APIRouter

router = APIRouter()


@router.get("/metrics")
async def get_metrics() -> dict[str, int]:
    """
    Get system metrics.
    
    Returns memory usage information including free and total memory.
    Uses psutil as the Python equivalent of Node.js os module.
    
    Returns:
        Dictionary containing memory metrics in bytes
    """
    memory = psutil.virtual_memory()

    return {
        "free": memory.available,
        "total": memory.total,
    }
