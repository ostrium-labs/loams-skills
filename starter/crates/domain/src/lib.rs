use serde::{Deserialize, Serialize};

pub type NoteId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInput {
    pub id: Option<NoteId>,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
}

impl NoteInput {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        if self.body.len() > 100_000 {
            return Err(DomainError::BodyTooLong);
        }
        if self.tags.len() > 32 {
            return Err(DomainError::TooManyTags);
        }
        if self
            .tags
            .iter()
            .any(|tag| tag.len() > 64 || tag.trim().is_empty())
        {
            return Err(DomainError::InvalidTag);
        }
        if let Some(embedding) = &self.embedding
            && embedding.len() != EMBEDDING_DIM
        {
            return Err(DomainError::InvalidEmbedding);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

impl SearchQuery {
    pub fn validate(&self) -> Result<(), DomainError> {
        if !(1..=100).contains(&self.limit) {
            return Err(DomainError::InvalidLimit);
        }
        if self.q.trim().is_empty() && self.tag.is_none() {
            return Err(DomainError::EmptyQuery);
        }
        Ok(())
    }
}

pub const EMBEDDING_DIM: usize = 4;

fn default_limit() -> u32 {
    10
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("body must not exceed 100000 bytes")]
    BodyTooLong,
    #[error("at most 32 tags are allowed")]
    TooManyTags,
    #[error("each tag must be 1..=64 bytes")]
    InvalidTag,
    #[error("embedding must have {EMBEDDING_DIM} dimensions")]
    InvalidEmbedding,
    #[error("search needs a query or a tag")]
    EmptyQuery,
    #[error("limit must be between 1 and 100")]
    InvalidLimit,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note() -> NoteInput {
        NoteInput {
            id: None,
            title: "Refund policy".into(),
            body: "thirty days".into(),
            tags: vec!["policy".into()],
            embedding: None,
        }
    }

    #[test]
    fn valid_note_passes() {
        assert!(note().validate().is_ok());
    }

    #[test]
    fn empty_title_fails() {
        let mut n = note();
        n.title = "  ".into();
        assert_eq!(n.validate(), Err(DomainError::EmptyTitle));
    }

    #[test]
    fn wrong_embedding_fails() {
        let mut n = note();
        n.embedding = Some(vec![1.0, 2.0]);
        assert_eq!(n.validate(), Err(DomainError::InvalidEmbedding));
    }

    #[test]
    fn limit_is_bounded() {
        let q = SearchQuery {
            q: "refund".into(),
            tag: None,
            limit: 0,
        };
        assert_eq!(q.validate(), Err(DomainError::InvalidLimit));
    }
}
