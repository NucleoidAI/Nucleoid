"""CLI interface for Nucleoid AI."""

import asyncio
import sys
from typing import Optional

import click
from colorama import Fore, Style

from . import nucleoid
from .datastore import DataStore


@click.group()
@click.version_option(version="0.7.10")
def main():
    """Nucleoid AI - Neuro-Symbolic AI runtime environment."""
    pass


@main.command()
@click.option("--id", help="Set id for the runtime", type=str)
@click.option("--clear", is_flag=True, help="Clear data before starting the runtime")
@click.option("--silence", is_flag=True, help="Silence the console")
@click.option("--debug", is_flag=True, help="Enable debug mode")
@click.option("--cluster", is_flag=True, help="Enable cluster mode")
@click.option("--terminal-port", help="Set terminal port number", type=int)
@click.option("--cluster-port", help="Set cluster port number", type=int)
def start(
    id: Optional[str],
    clear: bool,
    silence: bool,
    debug: bool,
    cluster: bool,
    terminal_port: Optional[int],
    cluster_port: Optional[int],
):
    """Start Nucleoid runtime."""
    try:
        config = {}
        
        if id:
            config["id"] = id
        
        if clear:
            datastore = DataStore()
            datastore.clear()
            if not silence:
                print("Data cleared")
        
        if terminal_port:
            if "port" not in config:
                config["port"] = {}
            config["port"]["terminal"] = terminal_port
        
        if cluster_port:
            if "port" not in config:
                config["port"] = {}
            config["port"]["cluster"] = cluster_port
        
        if debug:
            config["debug"] = debug
        
        if cluster:
            config["cluster"] = cluster
        
        # Start the runtime
        nucleoid.start(config)
        
        # Import and run the server
        from .server import run_server
        run_server(config)
        
    except KeyboardInterrupt:
        if not silence:
            print(f"\n{Fore.YELLOW}Shutting down Nucleoid runtime...{Style.RESET_ALL}")
        sys.exit(0)
    except Exception as e:
        if not silence:
            print(f"{Fore.RED}Error starting Nucleoid: {e}{Style.RESET_ALL}")
        sys.exit(1)


@main.command()
def clear():
    """Clear data."""
    try:
        datastore = DataStore()
        datastore.clear()
        print("Data is cleared")
    except Exception as e:
        print(f"{Fore.RED}Error clearing data: {e}{Style.RESET_ALL}")
        sys.exit(1)


@main.command()
@click.argument("statement")
@click.option("--details", is_flag=True, help="Show detailed execution information")
def run(statement: str, details: bool):
    """Run a Nucleoid statement."""
    try:
        result = nucleoid.run(statement, {"details": details})
        
        if details:
            click.echo(f"Statement: {statement}")
            click.echo(f"Result: {result}")
        else:
            click.echo(result)
            
    except Exception as e:
        print(f"{Fore.RED}Error running statement: {e}{Style.RESET_ALL}")
        sys.exit(1)


@main.command()
@click.option("--port", default=3000, help="Port to run the web server on")
def serve(port: int):
    """Start web server."""
    try:
        from .express import create_app
        
        app = create_app()
        app.listen(port, lambda: print(f"Server running on http://localhost:{port}"))
        
    except Exception as e:
        print(f"{Fore.RED}Error starting server: {e}{Style.RESET_ALL}")
        sys.exit(1)


if __name__ == "__main__":
    main()