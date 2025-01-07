# Rooch Common

The `rooch-common` crate provides shared utilities, functions, and abstractions used across the `Rooch` project. It acts
as a foundational library housing reusable components, making it easier to maintain consistency and reduce duplication
throughout the codebase.

## Features

This crate includes:

- **Filesystem-related utilities** (`src/fs`): Functions and abstractions for managing and interacting with the file
  system.
- **General-purpose utilities** (`src/utils`): A collection of helpful utility functions and tools that streamline
  common programming tasks.
- **Shared components and patterns**: Encapsulated logic that is widely used across different parts of the project.

The `rooch-common` crate is designed to serve as a practical toolkit, providing everything from general utilities to
more specific functionalities, aiming to improve code reuse and modularity in the `Rooch` ecosystem.

## Usage

To depend on `rooch-common`, you can include it in your `Cargo.toml` file:

```toml
[dependencies]
rooch-common = { workspace = true }
```

From there, you can access its utilities and abstractions to streamline your development process.

## Structure

The crate is organized into logical submodules and folders:

- **`fs/`**: Provides tools for file management and I/O operations.
- **`utils/`**: Contains general-purpose utilities to simplify coding practices.
- Additional modules and tools to support shared functionality across the project.

## Contribution

Contributions are welcome to improve functionality or expand shared tools. Feel free to open issues or submit pull
requests to enhance the usability and performance of the `rooch-common` crate.

---

This crate is an essential part of `Rooch`, focusing on maintaining quality, reliability, and reusability in the
project's common functionality.