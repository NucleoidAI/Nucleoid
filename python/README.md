# Nucleoid AI - Python Implementation

A Neuro-Symbolic AI runtime environment with declarative programming support for Python 3.13.

## Installation

```bash
pip install nucleoidai
```

## Quick Start

```python
import nucleoidai

# Start the runtime
nucleoidai.start()

# Run declarative statements
result = nucleoidai.run("let x = 5")
print(result)  # 5

# Register declarative functions
@nucleoidai.register
def calculate_total(items):
    """Declarative function for calculating totals"""
    return sum(item.price for item in items)
```

## Features

- **Declarative Runtime**: Process declarative statements and build knowledge graphs
- **Type Safety**: Full Python 3.13 type hints support
- **FastAPI Integration**: Built-in web server with OpenAPI documentation
- **Graph Processing**: Knowledge graph management and reasoning
- **Data Persistence**: Built-in data store with serialization
- **CLI Tools**: Command-line interface for development and deployment

## Development

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Format code
black .
isort .

# Type checking
mypy .

# Linting
flake8 .
```

## License

Apache-2.0