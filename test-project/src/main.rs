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
    
    // Check if number is even or odd
    for num in &doubled {
        if num % 2 == 0 {
            println!("{} is even", num);
        } else {
            println!("{} is odd", num);
        }
    }
    
    let message = "Hello, World!";
    let file_path = "output.txt";
    
    write_to_file(file_path, message)?;
    let content = read_from_file(file_path)?;
    
    println!("File content: {}", content);
    
    Ok(())
}

fn write_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn read_from_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}