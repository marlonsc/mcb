/// A simple calculator for testing semantic search
///
/// This module provides basic arithmetic operations and demonstrates
/// code that should be easily discoverable via semantic search.

/// Adds two numbers together
///
/// # Examples
///
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtracts b from a
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiplies two numbers
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divides a by b, returning None if b is zero
pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(divide(10, 0), None);
    }
}
