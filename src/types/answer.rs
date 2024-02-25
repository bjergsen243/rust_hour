use crate::types::question::QuestionId;
use serde::{Deserialize, Serialize};

/// Represents an answer to a question.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Answer {
    /// Unique identifier for the answer.
    pub id: AnswerId,
    /// Content of the answer.
    pub content: String,
    /// ID of the question this answer is associated with.
    pub question_id: QuestionId,
}

/// Represents a unique identifier for an answer.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnswerId(pub i32);

/// Used for creating new answers.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewAnswer {
    /// Content of the new answer.
    pub content: String,
    /// ID of the question this new answer is associated with.
    pub question_id: QuestionId,
}
