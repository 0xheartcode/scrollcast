/// A simple calculator library with basic operations
pub struct Calculator {
    pub result: f64,
}

impl Calculator {
    /// Create a new calculator with initial value of 0
    pub fn new() -> Self {
        Calculator { result: 0.0 }
    }
    
    /// Add a number to the current result
    pub fn add(&mut self, value: f64) -> &mut Self {
        self.result += value;
        self
    }
    
    /// Subtract a number from the current result
    pub fn subtract(&mut self, value: f64) -> &mut Self {
        self.result -= value;
        self
    }
    
    /// Multiply the current result by a number
    pub fn multiply(&mut self, value: f64) -> &mut Self {
        self.result *= value;
        self
    }
    
    /// Divide the current result by a number
    pub fn divide(&mut self, value: f64) -> Result<&mut Self, &'static str> {
        if value == 0.0 {
            Err("Cannot divide by zero")
        } else {
            self.result /= value;
            Ok(self)
        }
    }
    
    /// Get the current result
    pub fn get_result(&self) -> f64 {
        self.result
    }
    
    /// Reset the calculator to 0
    pub fn reset(&mut self) -> &mut Self {
        self.result = 0.0;
        self
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut calc = Calculator::new();
        
        calc.add(10.0)
            .subtract(3.0)
            .multiply(2.0);
            
        assert_eq!(calc.get_result(), 14.0);
    }
    
    #[test]
    fn test_division() {
        let mut calc = Calculator::new();
        calc.add(20.0);
        
        assert!(calc.divide(4.0).is_ok());
        assert_eq!(calc.get_result(), 5.0);
        
        assert!(calc.divide(0.0).is_err());
    }
}