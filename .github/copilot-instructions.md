# GitHub Copilot Project Instructions

This repository uses the Move language and Rust for Rooch blockchain development. Please follow these project-specific conventions and best practices when generating code, comments, or suggestions:

## 1. Code Style and Comments

- All code comments must be in English and use only ASCII characters.
- Use clear, descriptive names for all functions, variables, and types.
- Document non-trivial logic and public APIs with concise English comments.

## 2. Build and Test

- Use the Makefile for building and testing whenever possible:
  - `make build` — Build Rust (release) and all Move components.
  - `make test` — Run all Rust and Move tests.
  - `make lint` — Run all linters (including non-ASCII comment check).
  - `make quick-check` — Quick compilation check (Rust debug, rooch-framework).
  - `make build-move` — Build all core Move frameworks.
  - `make test-move` — Run all Move framework and example tests.
- For Move package development, you may also use:
  - `rooch move build -p <package_path>`
  - `rooch move test -p <package_path> [filter] [--ignore_compile_warnings]`

## 3. Move Language Patterns

- Entry functions must not use `Option<T>` as parameters.
- Use `public(friend)` for controlled module access, and declare friends in the module header.
- Define error codes as constants and use them directly in `assert!` statements.
- All Move account addresses must be derived from Bitcoin addresses using the provided APIs.
- Always retrieve the sender's validated Bitcoin address from context, never trust unvalidated arguments.

## 4. Testing

- Always initialize the test environment with `genesis::init_for_test()` in test functions.
- Use both success and failure tests, and mock context as needed.

## 5. Additional Rules

- Prefer composition over inheritance-like patterns.
- Adhere strictly to Move's ownership and borrowing rules.
- For more detailed and AI-specific rules, see the `.cursorrules` file in the project root.
