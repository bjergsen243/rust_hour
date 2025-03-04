use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a question in the system.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Question {
    /// Unique identifier for the question.
    pub id: QuestionId,
    /// Title of the question.
    pub title: String,
    /// Content of the question.
    pub content: String,
    /// Optional tags associated with the question.
    pub tags: Option<Vec<String>>,
}
/// Represents a unique identifier for a question.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuestionId(pub i32);

impl FromStr for QuestionId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i32>().map(QuestionId)
    }
}

/// Used for creating new questions.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct NewQuestion {
    /// Title of the new question.
    pub title: String,
    /// Content of the new question.
    pub content: String,
    /// Optional tags for the new question.
    pub tags: Option<Vec<String>>,
}
