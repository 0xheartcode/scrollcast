# Test Project

This is a **test project** for demonstrating the Git-to-PDF tool with various file types.

## Features

- `Rust` code with syntax highlighting
- `JSON` configuration files
- `Markdown` documentation
- Different code patterns and structures

## Code Examples

### Rust Function
```rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

### JSON Configuration
```json
{
  "api_key": "secret123",
  "endpoints": ["users", "posts", "comments"]
}
```

## Usage

Run the calculator:
```bash
cargo run --bin calculator
```

## Testing

Run tests with:
```bash
cargo test
```