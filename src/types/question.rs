use serde::{Deserialize, Serialize};

// Represents a question within the system.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Question {
    // Unique identifier for the question.
    pub id: QuestionId,
    // Title of the question.
    pub title: String,
    // Content of the question.
    pub content: String,
    // Optional list of tags associated with the question.
    pub tags: Option<Vec<String>>,
}
// Represents a unique identifier for a question.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionId(pub i32);

// Used for creating new questions within the system.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewQuestion {
    // Title of the new question.
    pub title: String,
    // Content of the new question.
    pub content: String,
    // Optional list of tags to associate with the new question.
    pub tags: Option<Vec<String>>,
}
