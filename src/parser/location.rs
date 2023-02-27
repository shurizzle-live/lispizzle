#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}
