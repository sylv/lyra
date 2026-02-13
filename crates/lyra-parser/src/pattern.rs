#[macro_export]
macro_rules! pattern {
    ($pattern:expr) => {
        Regex::new($pattern).unwrap()
    };
}
