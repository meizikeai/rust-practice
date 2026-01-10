# rust-practice

A practice project built with Rust, following clean architecture principles and common enterprise-level project structure.

## Tech Stack

- Web framework: [axum](https://github.com/tokio-rs/axum)
- Database: Supports [MySQL](https://github.com/launchbadge/sqlx)
- Cache: Supports [Redis](https://github.com/redis-rs/redis-rs)
- Runtime: Requires [Rust](https://rust-lang.org/) environment

## Project Structure

```bash
rust-practice
|-- src
|   |-- handler             → HTTP handler
|   |-- repository          → Data access layer
|   |-- utils               → Utility packages (common reusable functions)
|   |-- config.rs           → Configuration
|   |-- main.rs             → Application entry point
|   |-- router.rs           → Routing definition
|-- Cargo.mod               → Rust module and dependency management
|-- README.md               → Project documentation
```

## Quick Start

```bash
# Clone the repository
git clone https://github.com/meizikeai/rust-practice.git
cd rust-practice

# Run the application
cargo run
```

## Recommended Development Environment

- Editor: [Visual Studio Code](https://code.visualstudio.com)
- Rust extension and tools: Refer to the official [Rust in Visual Studio Code](https://code.visualstudio.com/docs/languages/rust)

Please make sure all Go tools are properly installed before starting development.

## Helpful Resources

- [The Rust Programming Language](https://rust-lang.org)
- [Rust Documentation](https://docs.rs)
- [Web Frameworks](https://www.arewewebyet.org/topics/frameworks)
