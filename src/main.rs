fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {

    #[test]
    fn smoke_test() {
        assert!(std::env::args().next().is_some());
    }
}
