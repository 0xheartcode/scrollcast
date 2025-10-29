// A simple Rust program for testing
fn main() {
    println!("Hello, Scrollcast!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    for n in numbers {
        println!("Number: {}", n);
    }
}

// A function with documentation
/// Adds two numbers together
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}