use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub struct TempDiff {
    temp_diff: i16,
}

impl Display for TempDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.temp_diff.is_positive() {
            true => write!(f, "+{}", self.temp_diff),
            false => write!(f, "{}", self.temp_diff),
        }
    }
}

impl TempDiff {
    pub fn new(temp_diff: &str) -> Self {
        Self {
            temp_diff: temp_diff.parse::<i16>().unwrap(),
        }
    }
}
