# Test Project

## src/main.rs

```rust
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};

// This is a test file to demonstrate syntax highlighting
fn main() -> Result<(), io::Error> {
    let mut scores = HashMap::new();
    scores.insert("Alice", 95);
    scores.insert("Bob", 87);
    scores.insert("Carol", 92);
    
    // Print all scores
    for (name, score) in &scores {
        println!("{}: {}", name, score);
    }
    
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
    
    Ok(())
}
```

## src/lib.rs

```rust
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```