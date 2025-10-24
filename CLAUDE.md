# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library implementing ICU MessageFormat version 1 (`platformed-mf`). The library provides internationalization (i18n) message formatting with parameter interpolation, pluralization, and localization support.

### Key Dependencies
- `nom` - Parser combinator library for parsing ICU MessageFormat strings
- `icu4x` crates - For localization of dates, numbers, and other locale-sensitive formatting
- Standard Rust testing framework for comprehensive unit tests

## Common Commands

### Building and Testing
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version
- `cargo test` - Run all tests (primary development workflow)
- `cargo test -- --nocapture` - Run tests with output visible
- `cargo test <test_name>` - Run specific test
- `cargo check` - Fast syntax and type checking without building
- `cargo clippy` - Run the Rust linter for code quality checks
- `cargo fmt` - Format code according to Rust standards

### Development
- `cargo doc --open` - Generate and open documentation
- `cargo clean` - Clean build artifacts

## Development Approach

### Test-Driven Development
All development follows unit testing practices:
1. Write tests first based on ICU MessageFormat examples
2. Implement functionality to pass tests
3. Refactor and optimize while maintaining test coverage

### Implementation Phases
1. **Basic Parameter Interpolation** - Simple `{param}` substitution
2. **Pluralization** - `{count, plural, one{...} other{...}}` patterns
3. **Select Statements** - `{gender, select, male{...} female{...} other{...}}`
4. **Number/Date Formatting** - Integration with ICU4X for locale-aware formatting
5. **Nested Messages** - Complex nested structure support

### Test Examples Sources
Tests include examples from:
- https://docs.tolgee.io/platform/translation_process/icu_message_format
- https://unicode-org.github.io/icu/userguide/format_parse/messages/

## Project Structure

- `src/lib.rs` - Main library entry point and core API
- `src/parser.rs` - nom-based ICU MessageFormat parser
- `src/formatter.rs` - Message formatting logic
- `src/types.rs` - Core data structures for parsed messages
- `tests/` - Integration tests with ICU MessageFormat examples
- `Cargo.toml` - Project configuration and dependencies

## Architecture Notes

The library follows a parse-then-format architecture:
1. Parse ICU MessageFormat strings into internal AST using nom
2. Provide formatting API that takes parsed messages and parameters
3. Delegate locale-specific formatting to ICU4X components
4. Support incremental complexity from basic interpolation to full ICU MessageFormat spec