"""
Command Line Interface for Nucleoid.

This module provides the CLI commands for starting and managing the Nucleoid runtime,
corresponding to the TypeScript bin.ts module.
"""

import asyncio

import click
from rich.console import Console

from .config import init as init_config
from .datastore import datastore

console = Console()


@click.group()
@click.version_option(version="0.7.10")
def cli():
    """Nucleoid - Declarative, logic-based, contextual runtime for Neuro-Symbolic AI."""
    pass


@cli.command()
@click.option("--id", help="Set id for the runtime", type=str)
@click.option("--clear", is_flag=True, help="Clear data before starting the runtime")
@click.option("--silence", is_flag=True, help="Silence the console output")
@click.option("--debug", is_flag=True, help="Enable debug mode")
@click.option("--cluster", is_flag=True, help="Enable cluster mode")
@click.option("--terminal-port", help="Set terminal port number", type=int)
@click.option("--cluster-port", help="Set cluster port number", type=int)
def start(
    id: str | None = None,
    clear: bool = False,
    silence: bool = False,
    debug: bool = False,
    cluster: bool = False,
    terminal_port: int | None = None,
    cluster_port: int | None = None,
):
    """Start Nucleoid runtime."""

    # Prepare CLI arguments for config
    cli_args = {}
    if id:
        cli_args["id"] = id
    if terminal_port:
        cli_args["terminal_port"] = terminal_port
    if cluster_port:
        cli_args["cluster_port"] = cluster_port

    # Clear data if requested
    if clear:
        datastore.clear()
        if not silence:
            console.print("Data cleared before starting", style="yellow")

    # Initialize configuration with CLI args
    config = {}
    init_config(config, cli_args=cli_args)

    # Import and start the server
    try:
        asyncio.run(_start_server(silence, debug, cluster))
    except KeyboardInterrupt:
        if not silence:
            console.print("\n🌿 Nucleoid runtime stopped", style="bright_green")


@cli.command()
def clear():
    """Clear stored data."""
    datastore.clear()
    console.print("Data is cleared", style="green")


async def _start_server(silence: bool = False, debug: bool = False, cluster: bool = False):
    """Start the Nucleoid server."""
    import uvicorn

    from . import start as nucleoid_start
    from .app import app

    # Start the runtime
    if not silence:
        nucleoid_start()

    # Configure uvicorn server
    config = uvicorn.Config(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="debug" if debug else "info",
        access_log=not silence
    )

    # Start the web server
    server = uvicorn.Server(config)

    if not silence:
        console.print("🌐 Web server starting on http://0.0.0.0:8000", style="bright_blue")
        console.print("Press Ctrl+C to stop the runtime", style="dim")

    try:
        await server.serve()
    except KeyboardInterrupt:
        pass


def main():
    """Main CLI entry point."""
    cli()
