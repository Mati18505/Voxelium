# AGENTS.md

## Project Overview
**Voxel engine made with Rust and Bevy.**

Its main goals are performance and ease of extensibility.

## Workspace Structure
- `client/` - Rendering client using Bevy
- `server/` - Server component (minimal at the moment)
- `shared/` - Shared types, entities, physics, and chunk I/O

## Commands

### Running Tests
- Run all tests: `cargo test`
- Run a single test: `cargo test <test_name>`
- Run tests in a specific package: `cargo test -p <package_name>`
- Run tests with output: `cargo test -- --nocapture`

### Building
- Build client: `cargo build -p client`
- Build all: `cargo build`
- Build with release profile: `cargo build -p client --release`

### Code Quality
- Clippy (linting): `cargo clippy --all-targets`
- Clippy with fixes: `cargo clippy --all-targets --fix --allow-dirty`
- Format code: `cargo fmt`
- Check formatting: `cargo fmt --check`

## Code Style Guidelines

### General Principles
- Prioritize readability and maintainability
- Use Rust's type system to prevent invalid states
- Avoid premature optimization; profile first

### Naming Conventions
- **Types/Structs/Enums**: `PascalCase` (e.g., `ChunkPos`, `BlockInChunkPos`)
- **Functions/Methods**: `snake_case` (e.g., `is_within_distance`, `get_block`)
- **Variables**: `snake_case` (e.g., `block_pos`, `chunk_manager`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `CHUNK_SIZE`)
- **Type Aliases**: `PascalCase` ending in descriptive suffix (e.g., `BlockID`, `BlockPos`, `MaterialName`)
- **Modules**: `snake_case` (e.g., `chunk_io`, `physics`)
- **Crates**: `snake_case` (e.g., `client`, `shared`)

### Imports
- Use absolute imports from crate root for intra-crate modules: `use crate::module::Item;`
- Use external crate imports directly: `use cgmath::Vector3;`
- Group std, external, and crate imports (optional but recommended)
- Use glob imports sparingly, prefer specific imports

### Structs and Enums
- Use `#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]` as appropriate for data types
- Use `#[derive(Default)]` when a zero-value default makes sense
- Prefer newtype wrappers (`struct Wrapper(InnerType)`) over type aliases for invariants
- Use tuple structs for minimal data containers: `pub struct ChunkPos(Vector3<isize>);`
- Use regular structs with named fields when behavior or documentation is needed

### Error Handling
- Use `thiserror` for defining custom error types: `#[derive(Debug, Error)]`
- Use `#[error("...")]` macro for error messages
- Prefer `Result<T, E>` for fallible operations
- Use `?` operator for error propagation
- Use `assert!` with descriptive messages for invariant violations

### Documentation
- Document public APIs with `///` doc comments
- Add comments explaining *why*, not *what*
- Include examples in doc comments for important functions
- Document assumptions and panics in function signatures

### Testing
- Place tests in `#[cfg(test)]` modules within the file being tested
- Name test functions descriptively: `test_<function>_<scenario>`
- Use `#[test]` attribute for unit tests
- Use assertions with informative messages: `assert!(condition, "message")`

### Bevy-Specific Patterns
- Use Bevy plugins for feature organization: `impl Plugin for MyPlugin`
- Systems use `fn system_name(mut queries: Query<...>)`
- Resources use `#[derive(Resource)]`
- Events use `#[derive(Event)]`
- Assets use `#[derive(Asset)]`
- Use `bevy::prelude::*` for quick prototyping, specific imports in public APIs
- Follow Bevy's component/query ordering: Resources → Queries → Commands

### Performance Considerations
- Use `Deref`/`DerefMut` for newtype wrappers to avoid delegation boilerplate
- Use `Option<T>` for nullable values instead of null pointers
- Use iterators over loops when appropriate
- Consider `copy` vs` implications `clone for large types
- Use `#[inline]` for small, frequently called functions

### Formatting
- Run `cargo fmt` before committing
- Follow Rust standard formatting (4 spaces, 100 char max line length)
- Use trailing commas in multiline function calls

### Common Patterns in This Codebase
- Newtype pattern with `Deref` for position types (`ChunkPos`, `BlockInChunkPos`)
- `BlockID = u8` for block identifiers
- `BlockPos = Vector3<isize>` for world positions
- `CHUNK_SIZE: usize = 16` for chunk dimensions
- `assert!` for validating constructor preconditions
- `checked_*` and `*_unchecked` method naming for safe vs unsafe operations

### Anti-Patterns to Avoid
- Don't use `println!` for debugging in production code (use `tracing` or `log`)
- Don't expose internal mutability without justification
- Don't ignore `Result` values with `_`
- Don't use `unsafe` without proper justification and documentation

## Collaboration Rule
- Do not modify source code by default.
- Only make code changes after the user clearly and explicitly asks for implementation/editing (e.g. "zrób", "zaimplementuj", "zmień kod", "apply patch").
- If the request is ambiguous or sounds like analysis/review/question, provide explanation first and wait for explicit confirmation before editing files.
