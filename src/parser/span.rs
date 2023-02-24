use super::Location;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: Location,
    pub stop: Location,
}
