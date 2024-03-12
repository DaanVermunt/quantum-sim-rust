fn main() -> u8 {
    println!("Hello, world!");
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        assert_eq!(main(), 0);
    }
}