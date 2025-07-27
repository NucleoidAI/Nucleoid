# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nucleoid is a declarative, logic-based, contextual runtime for Neuro-Symbolic AI. It's a symbolic AI implementation that manages both data and logic under the same runtime environment, creating a knowledge graph that tracks relationships between declarative statements.

## Core Architecture

### Main Components

- **Runtime (`src/nucleoid.ts`)**: Core runtime functions including `start()`, `run()`, and `register()`
- **Expression Engine (`src/Expression.ts`)**: Handles expression evaluation and processing
- **Instruction System (`src/Instruction.ts`)**: Manages instruction execution
- **Scope Management (`src/Scope.ts`)**: Handles variable and function scoping
- **Context (`src/context.ts`)**: Manages execution context and state
- **Language Support (`src/lang/`)**: Contains language-specific implementations including AST handling and evaluation
- **NUC Modules (`src/nuc/`)**: Core language constructs (ALIAS, BLOCK, CLASS, FUNCTION, etc.)

### Key Directories

- `src/lang/$nuc/`: Core language runtime components
- `src/lang/ast/`: Abstract Syntax Tree handling
- `src/lang/estree/`: ECMAScript tree parsing and generation
- `src/nuc/`: Language construct implementations
- `src/routes/`: API endpoints (graph, logs, metrics, openapi, terminal)
- `src/lib/`: Utility libraries (deep, openapi, random, serialize, statement, test)

## Development Commands

### Building and Running
```bash
# Start the runtime
npm start

# Start with CLI options
node bin.js start [--id <id>] [--clear] [--silence] [--debug] [--cluster]
```

### Testing
```bash
# Run all tests
npm test

# Tests use Jest with TypeScript preset
# Test files follow pattern: **/*.spec.ts
```

### Code Quality
```bash
# Run linting
npm run lint
```

## Configuration

- **TypeScript**: Configured for ES Next with Node.js modules
- **Jest**: Uses ts-jest preset for TypeScript testing
- **ESLint**: Configured with TypeScript support and Prettier integration

## Key Features

- **Declarative Programming**: Uses 'use declarative' pragma for logic statements
- **Dynamic Knowledge Graph**: Creates relationships between data and logic statements
- **Multi-language Support**: TypeScript (Beta), Python (WiP)
- **Built-in Datastore**: Uses @nucleoidjs/datastore for persistence
- **Express Integration**: Web framework integration via `src/express.ts`
- **OpenAPI Support**: Automatic API documentation generation

## Binary Commands

The project provides three CLI aliases:
- `nucleoidai`
- `nucleoid` 
- `nuc`

All point to `bin.js` and support the `start` command with various options for runtime configuration.

## Testing Structure

Tests are organized alongside source files with `.spec.ts` extension. Key test areas include:
- AST processing (`src/lang/ast/test/`)
- Route handlers (`src/routes/test/`)
- Core functionality (`src/test/`)
- Library utilities (`src/lib/test/`)