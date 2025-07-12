# TrekBot Project Structure

## Overview
TrekBot is a Rust implementation of a player for the Super Star Trek game that can work with any BASIC interpreter by launching it as a subprocess and communicating via stdin/stdout.

## Architecture

### Core Components

1. **Interpreter** - Interface for launching and communicating with different BASIC interpreters
2. **GameState** - Parses game output and tracks current state
3. **Strategy** - Defines different playing strategies (Random, Cheat, etc.)
4. **Player** - Orchestrates the game by connecting interpreter, state, and strategy

### Module Structure

```
src/
├── main.rs              # CLI interface and main entry point
├── interpreter/
│   ├── mod.rs          # Interpreter trait and common functionality
│   ├── basicrs.rs      # BasicRS interpreter implementation
│   ├── trekbasic.rs    # Python TrekBasic interpreter implementation
│   └── trekbasicj.rs   # Java TrekBasicJ interpreter implementation
├── game/
│   ├── mod.rs          # Game module exports
│   ├── state.rs        # Game state parsing and tracking
│   └── parser.rs       # Output parsing utilities
├── strategy/
│   ├── mod.rs          # Strategy trait and common functionality
│   ├── random.rs       # Random strategy implementation
│   └── cheat.rs        # Intelligent cheat strategy implementation
└── player.rs           # Main player orchestration
```

## Key Design Principles

1. **Interpreter Independence** - No internal APIs, only subprocess communication
2. **Strategy Pattern** - Different playing strategies can be swapped
3. **Error Handling** - Robust error handling for subprocess communication
4. **Testability** - All components are unit testable
5. **Extensibility** - Easy to add new interpreters or strategies
