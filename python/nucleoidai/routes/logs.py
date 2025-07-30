"""Logs API routes for Nucleoid AI."""

from typing import List, Dict, Any
from fastapi import APIRouter, Request

from ..datastore import DataStore

router = APIRouter()
datastore = DataStore()


@router.get("/logs")
async def get_logs(request: Request) -> List[Dict[str, Any]]:
    """Get recent execution logs."""
    logs = datastore.tail()
    
    # Convert Data objects to dictionaries for JSON serialization
    result = []
    for log in logs:
        if hasattr(log, '__dict__'):
            log_dict = log.__dict__.copy()
            # Convert datetime to string if present
            if 'date' in log_dict and log_dict['date']:
                log_dict['date'] = log_dict['date'].isoformat()
            result.append(log_dict)
        else:
            result.append(log)
    
    return result