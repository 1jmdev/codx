# RULES.md

## Agent Behavior

- **ALWAYS USE PARALLEL TOOLS WHEN APPLICABLE.**
- Implement everything the user asks for. Never leave a feature incomplete, stubbed out, or partially implemented. Every request must be fully delivered.
- Do not interact with git (no commits, no pushes, no branch operations) unless explicitly told to.
- Do not run tests unless explicitly told to.
- Do not run formatting commands (`cargo fmt`, etc.).
- If no LSP diagnostics are available, run `cargo check` to verify correctness.
- Do not add, remove, or modify code unrelated to the current task.

---

## Project Structure

### File Organization

- Keep files small and focused on a single responsibility. A file should represent one logical unit (one struct + its impl, one trait, one module of related functions, etc.).
- The only exception to file size limits are registry-style files (e.g., macro registries, enum variant listings, route tables).

### Module Files (`mod.rs` / `lib.rs` / `main.rs`)

- `mod.rs`, `lib.rs`, and `main.rs` must contain **only** module declarations (`mod`) and re-exports (`pub use`).
- **Never** place logic, struct definitions, trait implementations, helper functions, or tests in these files.
- All functionality lives in dedicated submodule files.

```text
src/
├── main.rs              // only: mod declarations, entry point call
├── app/
│   ├── mod.rs           // only: mod + pub use
│   ├── startup.rs
│   └── config.rs
├── handlers/
│   ├── mod.rs           // only: mod + pub use
│   ├── auth.rs
│   └── users.rs
├── models/
│   ├── mod.rs           // only: mod + pub use
│   ├── user.rs
│   └── session.rs
└── services/
    ├── mod.rs           // only: mod + pub use
    ├── auth_service.rs
    └── user_service.rs
```

### Tests

- Tests live in a `tests/` submodule or in `tests/` at the crate root — never inside `mod.rs` / `lib.rs` / `main.rs`.

---

## Error Handling

- **Never** use `.unwrap()` or `.expect()`. The binary must not panic on recoverable errors.
- Use `Result<T, E>` for all fallible operations. Propagate errors with `?`.
- Define domain-specific error enums. Implement `std::error::Error` or use `thiserror` for library code.
- For application-level error propagation, prefer `thiserror` or a crate like `anyhow` only at the binary boundary.
- Convert foreign errors at module boundaries into your own error types.

```rust
// Good
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

// Forbidden
let config: Config = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
```

---

## Ownership & Borrowing

- **Never** silence the borrow checker with `.clone()`. If the code doesn't compile, redesign the data flow.
- Prefer borrowing (`&T`, `&mut T`) over owned values whenever the callee does not need ownership.
- Use `Cow<'_, T>` when a function may or may not need to allocate.
- Prefer `&str` over `String`, `&[T]` over `Vec<T>`, and `&Path` over `PathBuf` in function parameters.
- Return owned types (`String`, `Vec<T>`) only when the function genuinely produces new data.

---

## Concurrency

- Avoid locks (`Mutex`, `RwLock`) whenever possible. Prefer:
  - Message passing (`tokio::sync::mpsc`, `flume`, `crossbeam::channel`).
  - Lock-free types (`AtomicU64`, `AtomicBool`, `arc-swap`).
  - Task-local ownership — structure the system so each piece of mutable state is owned by exactly one task.
- When a lock is truly needed, prefer `RwLock` over `Mutex` for read-heavy workloads.
- Never hold a lock across an `.await` point (use `tokio::sync::Mutex` only if unavoidable, and minimize the critical section).
- Prefer `Arc` with immutable data + atomic interior state over `Arc<Mutex<T>>`.

---

## Performance

### Allocation

- Avoid unnecessary heap allocations. Prefer stack-allocated types and slices.
- Use `&str` instead of `String`, `&[T]` instead of `Vec<T>` whenever the data is borrowed.
- Use `SmallVec` or `ArrayVec` when the common case fits a known small size.
- Use `Box<T>` only when you need heap allocation for a concrete reason (trait objects, recursive types, large structs).

### Iterators

- Use iterator chains instead of manual `for` loops with manual accumulation. Iterators are zero-cost and compose better.

```rust
// Good
let total: u64 = items.iter().filter(|i| i.active).map(|i| i.value).sum();

// Avoid
let mut total: u64 = 0;
for item in &items {
    if item.active {
        total += item.value;
    }
}
```

### Pre-allocation

- When the size of a collection is known or estimable, pre-allocate:

```rust
let mut results = Vec::with_capacity(inputs.len());
```

- Same applies to `String::with_capacity`, `HashMap::with_capacity`, etc.

### String Building

- Avoid `format!()` for simple concatenation. Prefer `push_str` or direct construction.

```rust
// Good — no format machinery
let mut path = base.to_string();
path.push('/');
path.push_str(segment);

// Acceptable — complex formatting
let msg = format!("{name} ({id}): {status}");

// Avoid — trivial concat through format
let path = format!("{}{}", base, segment);
```

### General

- Prefer `&[u8]` / byte-level operations over string operations when working with protocols or binary data.
- Use `#[inline]` only on small, hot, cross-crate functions. Do not blanket-inline.
- Prefer `into_iter()` over `iter().cloned()` when consuming a collection.
- Avoid `collect()`-ing into an intermediate `Vec` only to iterate again. Chain the iterators.

---

## Type Design

- Use newtypes to enforce domain invariants at compile time (`struct UserId(u64)`).
- Use enums over boolean flags when a value has more than two semantic states.
- Derive only the traits you need. Do not blanket `#[derive(Clone, Debug, PartialEq, Eq, Hash)]` on everything.
- Keep structs small. If a struct has more than ~5–6 fields, consider splitting it into sub-structs.
- Use `builder` patterns or dedicated config structs for complex construction instead of functions with many parameters.

---

## API Design

- Public functions must have clear, minimal signatures. Accept the most general input type and return the most specific output type.
- Use `impl Into<T>` / `impl AsRef<T>` on public APIs sparingly and only when ergonomic benefit is clear.
- Prefer returning `impl Iterator<Item = T>` over `Vec<T>` when the caller may not need all items.
- Keep `pub` surface area minimal. Default to private; expose only what the module boundary needs.

---

## Dependencies

- Do not add dependencies without explicit user approval.
- Prefer well-maintained, minimal crates. Avoid pulling in large frameworks for small tasks.
- Pin dependency versions in `Cargo.toml` (e.g., `serde = "1.0.210"`, not `serde = "1"`).

---

## Code Quality Checklist (Self-Review Before Completion)

1. No `.unwrap()` or `.expect()` calls.
2. No `.clone()` used to bypass borrow checker.
3. No logic in `mod.rs` / `lib.rs` / `main.rs`.
4. No dead code, commented-out blocks, or TODO stubs.
5. All requested features fully implemented.
6. `cargo check` passes (when LSP is unavailable).
7. No unnecessary allocations; iterators used where appropriate.
8. No locks where message passing or atomics suffice.
9. Error types properly defined and propagated.
10. Files are small and single-purpose.