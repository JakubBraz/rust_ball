pub mod physics;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn f2(a: u32, b: u32) -> u32{
    a * b
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
