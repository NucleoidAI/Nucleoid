# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nucleoid is a Neuro-Symbolic AI project with a declarative (logic) runtime environment for TypeScript. The project implements a knowledge graph-based reasoning engine that combines symbolic logic with neural networks.

### Architecture

- **Main TypeScript implementation**: Located in `typescript/` directory
- **Core runtime**: `src/nucleoid.ts` - Main entry point with start(), run(), and register() functions
- **Language processing**: `src/lang/` - Contains AST parsing, evaluation, and declarative syntax handling
- **Declarative constructs**: `src/nuc/` - Implementation of declarative programming constructs (CLASS, OBJECT, FUNCTION, etc.)
- **Express integration**: `src/express.ts` - Web server integration
- **Data store**: `src/datastore.ts` - Built-in data persistence
- **Graph processing**: `src/graph.ts` - Knowledge graph management
- **Runtime execution**: `src/runtime.ts` - Core statement processing engine

### Key Components

- **Declarative Runtime**: Processes declarative statements and builds knowledge graphs
- **AST Processing**: Located in `src/lang/ast/` - Handles expression parsing and conversion
- **ESTree Integration**: `src/lang/estree/` - JavaScript AST parsing and generation
- **Statement Types**: `src/nuc/` contains implementations for different declarative constructs
- **Routes**: `src/routes/` - API endpoints for graph, logs, metrics, OpenAPI, and terminal

## Development Commands

### Working Directory
All commands should be run from the `typescript/` directory.

### Core Commands
- **Install dependencies**: `npm install`
- **Run tests**: `npm test` (uses Jest with TypeScript preset)
- **Lint code**: `npm run lint` (ESLint with TypeScript support)
- **Start server**: `npm start` or `node bin.js start`

### Testing
- Test files use `.spec.ts` extension
- Jest configuration in `jest.config.json` with `ts-jest` preset
- Tests located alongside source files in various `test/` subdirectories

### TypeScript Configuration
- Target: `esnext` with `nodenext` module system
- Strict mode enabled with `noImplicitAny: false`
- ES module interop enabled

## Key Files and Their Purpose

- `index.ts` - Main export file exposing nucleoid, express, config, and utility functions
- `bin.ts` - CLI entry point
- `server.ts` - Basic server startup
- `src/nucleoid.ts` - Core runtime functions (start, run, register)
- `src/runtime.ts` - Statement processing engine
- `src/context.ts` - Runtime context management
- `src/config.ts` - Configuration management
- `src/types.ts` - TypeScript type definitions

## Branch Information
- **Current branch**: `python-claude-code`
- **Main branch**: `typescript` (use for PRs)
- Repository has both Python and TypeScript implementations, with TypeScript being the primary focus

## Project Status
This is a beta-stage project implementing declarative logic programming for Neuro-Symbolic AI applications. The codebase includes comprehensive test coverage and follows TypeScript best practices.