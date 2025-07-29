# Nucleoid AI Python Development Makefile

.PHONY: install dev test lint format type-check clean build publish help

# Default target
help:
	@echo "Available targets:"
	@echo "  install     - Install package in production mode"
	@echo "  dev         - Install package in development mode"
	@echo "  test        - Run tests with pytest"
	@echo "  lint        - Run flake8 linting"
	@echo "  format      - Format code with black and isort"
	@echo "  type-check  - Run mypy type checking"
	@echo "  clean       - Clean build artifacts"
	@echo "  build       - Build package"
	@echo "  publish     - Publish to PyPI"
	@echo "  all         - Run format, lint, type-check, and test"

# Install package in production mode
install:
	pip install .

# Install package in development mode with dev dependencies
dev:
	pip install -e ".[dev]"
	pre-commit install

# Run tests
test:
	pytest -v --cov=nucleoidai --cov-report=term-missing

# Run linting
lint:
	flake8 src/ tests/

# Format code
format:
	black src/ tests/
	isort src/ tests/

# Type checking
type-check:
	mypy src/nucleoidai

# Clean build artifacts
clean:
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	find . -type d -name __pycache__ -delete
	find . -type f -name "*.pyc" -delete

# Build package
build: clean
	python -m build

# Publish to PyPI (requires authentication)
publish: build
	python -m twine upload dist/*

# Run all checks
all: format lint type-check test

# Development server
serve:
	nucleoidai serve --port 3000

# Start runtime
start:
	nucleoidai start