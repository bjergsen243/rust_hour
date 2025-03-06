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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_id_from_str_valid() {
        let id_str = "42";
        let question_id = QuestionId::from_str(id_str);
        assert!(question_id.is_ok());
        assert_eq!(question_id.unwrap(), QuestionId(42));
    }

    #[test]
    fn test_question_id_from_str_invalid() {
        let id_str = "not_a_number";
        let question_id = QuestionId::from_str(id_str);
        assert!(question_id.is_err());
    }

    #[test]
    fn test_question_id_from_str_negative() {
        let id_str = "-42";
        let question_id = QuestionId::from_str(id_str);
        assert!(question_id.is_ok());
        assert_eq!(question_id.unwrap(), QuestionId(-42));
    }

    #[test]
    fn test_question_id_from_str_zero() {
        let id_str = "0";
        let question_id = QuestionId::from_str(id_str);
        assert!(question_id.is_ok());
        assert_eq!(question_id.unwrap(), QuestionId(0));
    }

    #[test]
    fn test_question_id_from_str_max() {
        let id_str = i32::MAX.to_string();
        let question_id = QuestionId::from_str(&id_str);
        assert!(question_id.is_ok());
        assert_eq!(question_id.unwrap(), QuestionId(i32::MAX));
    }

    #[test]
    fn test_question_id_from_str_min() {
        let id_str = i32::MIN.to_string();
        let question_id = QuestionId::from_str(&id_str);
        assert!(question_id.is_ok());
        assert_eq!(question_id.unwrap(), QuestionId(i32::MIN));
    }

    #[test]
    fn test_question_id_from_str_overflow() {
        let id_str = "2147483648"; // i32::MAX + 1
        let question_id = QuestionId::from_str(id_str);
        assert!(question_id.is_err());
    }

    #[test]
    fn test_question_struct() {
        let question = Question {
            id: QuestionId(1),
            title: "Test Question".to_string(),
            content: "Test Content".to_string(),
            tags: Some(vec!["test".to_string()]),
        };

        assert_eq!(question.id, QuestionId(1));
        assert_eq!(question.title, "Test Question");
        assert_eq!(question.content, "Test Content");
        assert_eq!(question.tags, Some(vec!["test".to_string()]));
    }

    #[test]
    fn test_new_question_struct() {
        let new_question = NewQuestion {
            title: "Test Question".to_string(),
            content: "Test Content".to_string(),
            tags: Some(vec!["test".to_string()]),
        };

        assert_eq!(new_question.title, "Test Question");
        assert_eq!(new_question.content, "Test Content");
        assert_eq!(new_question.tags, Some(vec!["test".to_string()]));
    }
}
