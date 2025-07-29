"""Server module for Nucleoid AI."""

import asyncio
from typing import Dict, Any

from . import nucleoid


def run_server(config: Dict[str, Any]) -> None:
    """Run the Nucleoid server."""
    print("Server is running")
    
    # Keep the server running
    try:
        # In a real implementation, this might start additional services
        # For now, we'll just keep the process alive
        asyncio.get_event_loop().run_forever()
    except KeyboardInterrupt:
        print("Server stopped")


if __name__ == "__main__":
    nucleoid.start()
    run_server({})