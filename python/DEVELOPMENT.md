# Nucleoid AI Python Development Guide

## Project Structure

```
python/
├── nucleoidai/                 # Main package (following best practices)
│   ├── __init__.py            # Package initialization
│   ├── nucleoid.py            # Core runtime functions
│   ├── runtime.py             # Runtime processor
│   ├── context.py             # Context management
│   ├── config.py              # Configuration system
│   ├── datastore.py           # Data persistence
│   ├── cache.py               # Caching system
│   ├── event.py               # Event management
│   ├── transaction.py         # Transaction support
│   ├── stack.py               # Stack processing
│   ├── graph.py               # Knowledge graph
│   ├── state.py               # State management
│   ├── express.py             # FastAPI integration
│   ├── server.py              # Server management
│   ├── cli.py                 # Command-line interface
│   ├── types.py               # Type definitions
│   ├── lang/                  # Language processing
│   │   ├── __init__.py        # Language module exports
│   │   ├── evaluation.py      # Evaluation expressions
│   │   ├── ast/               # AST nodes
│   │   │   ├── identifier.py  # Identifier nodes
│   │   │   └── __init__.py    
│   │   ├── dollar_nuc/        # Dollar ($) constructs
│   │   │   ├── base.py        # Base dollar construct
│   │   │   ├── alias.py       # $ALIAS construct
│   │   │   ├── expression.py  # $EXPRESSION construct
│   │   │   └── __init__.py    
│   │   ├── estree/            # ESTree parser
│   │   │   ├── parser.py      # JavaScript-like syntax parser
│   │   │   └── __init__.py    
│   │   └── nuc/               # Language constructs (legacy)
│   │       ├── assignment.py  # Assignment constructs
│   │       ├── block.py       # Block constructs
│   │       ├── class_declaration.py # Class constructs
│   │       └── ... (other constructs)
│   ├── nuc/                   # Main declarative constructs
│   │   ├── __init__.py        # NUC module exports
│   │   ├── node.py            # Base NODE class
│   │   ├── class_construct.py # CLASS construct
│   │   ├── alias_construct.py # ALIAS construct
│   │   └── __init__.py        
│   ├── lib/                   # Utility libraries
│   │   ├── deep.py            # Deep object operations
│   │   ├── openapi.py         # OpenAPI integration
│   │   ├── test.py            # Testing utilities
│   │   ├── random.py          # Random utilities
│   │   ├── serialize.py       # Serialization
│   │   ├── statement.py       # Statement compilation
│   │   └── __init__.py        
│   └── routes/                # API routes
│       ├── graph.py           # Graph API
│       ├── logs.py            # Logs API
│       └── __init__.py        
├── tests/                     # Test suite
├── pyproject.toml             # Package configuration
├── Makefile                   # Development commands
├── README.md                  # Documentation
├── DEVELOPMENT.md             # Development guide
├── test_installation.py       # Installation verification
├── .flake8                    # Linting configuration
├── .pre-commit-config.yaml    # Pre-commit hooks
└── .gitignore                 # Git ignore rules
```

## Key Features

### ✅ Core Runtime
- **start()**: Initialize the Nucleoid runtime
- **run()**: Execute declarative statements
- **register()**: Register declarative functions
- Transaction support with rollback
- Event management and logging

### ✅ Type Safety
- Full Python 3.13 type hints
- Comprehensive type definitions
- MyPy compatible

### ✅ Web Framework
- FastAPI integration (replacing Express.js)
- REST API endpoints
- OpenAPI documentation support
- CORS and middleware

### ✅ Language Processing
- ESTree-compatible parser
- AST processing and compilation
- Declarative language constructs
- JavaScript/TypeScript-like syntax support

### ✅ Data Management
- Built-in data store with caching
- Knowledge graph processing
- Serialization and persistence
- Transaction management

### ✅ Development Tools
- CLI with multiple commands
- Comprehensive test suite (pytest)
- Code formatting (black, isort)
- Linting (flake8)
- Type checking (mypy)
- Pre-commit hooks

## Development Commands

```bash
# Install in development mode
make dev

# Run tests
make test

# Format code
make format

# Run linting
make lint

# Type checking
make type-check

# Run all checks
make all

# Start runtime
make start

# Start web server
make serve
```

## Usage Examples

### Basic Usage
```python
import nucleoidai

# Start the runtime
nucleoidai.start()

# Run statements
result = nucleoidai.run("x = 42")
print(result)  # 42

# Register declarative functions
@nucleoidai.register
def calculate_total(items):
    return sum(item.price for item in items)
```

### Web Application
```python
from nucleoidai import create_app

app = create_app()

@app.get("/health")
def health_check():
    return {"status": "healthy"}

app.listen(3000)
```

### CLI Usage
```bash
# Start runtime
nucleoidai start

# Run statement
nucleoidai run "x = 5"

# Start web server
nucleoidai serve --port 3000

# Clear data
nucleoidai clear
```

## Architecture Notes

The Python implementation maintains full compatibility with the TypeScript version while leveraging Python's strengths:

1. **Best Practices**: Package structure follows Python standards
2. **Type Safety**: Full type hints for Python 3.13
3. **Modern Tooling**: Black, flake8, mypy, pytest
4. **Web Framework**: FastAPI instead of Express.js
5. **CLI**: Click-based command-line interface
6. **Testing**: Comprehensive pytest test suite

## Installation Verification

Run the installation test to verify everything works:

```bash
python test_installation.py
```

This will test:
- Package imports
- Basic runtime functionality  
- Web application creation

## Contributing

1. Install development dependencies: `make dev`
2. Run tests: `make test`
3. Format code: `make format`
4. Run linting: `make lint`
5. Type check: `make type-check`

The project uses pre-commit hooks to ensure code quality.