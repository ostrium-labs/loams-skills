use connectrpc::client::HttpClient;
use domain::{NoteInput, SearchQuery};
use loams::builders::{DocId, Filter, Schema, Search, TextRetriever};
use loams::collection_v1::document_id;
use loams::collection_v1::{
    CreateCollectionRequest, CreateNamespaceRequest, DocumentId, GetCollectionRequest,
    GetDocumentsRequest, WriteDocumentsRequest,
};
use loams::{CallOptionsOverrides, Document, ErrorKind, Loams, LoamsError};
use serde_json::{Value, json};

use crate::Config;

#[derive(Clone)]
pub struct Platform {
    client: Loams<HttpClient>,
    namespace: String,
    collection: String,
}

impl std::fmt::Debug for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Platform")
            .field("namespace", &self.namespace)
            .field("collection", &self.collection)
            .finish_non_exhaustive()
    }
}

impl Platform {
    pub async fn connect(config: &Config) -> Result<Self, PlatformError> {
        let client = Loams::from_env()?.session_consistency(true);
        let platform = Self {
            client,
            namespace: config.namespace.clone(),
            collection: config.collection.clone(),
        };
        platform.ensure_schema().await?;
        Ok(platform)
    }

    async fn ensure_schema(&self) -> Result<(), PlatformError> {
        let created = self
            .client
            .namespaces()
            .create(
                CreateNamespaceRequest {
                    namespace: self.namespace.clone(),
                    ..Default::default()
                },
                opts(),
            )
            .await;
        ignore_existing(created.map(|_| ()))?;

        let schema = Schema::new()
            .text("title")
            .text("body")
            .keyword("tags")
            .vector("embedding", 4);
        let created = self
            .client
            .collections()
            .create(
                CreateCollectionRequest {
                    namespace: self.namespace.clone(),
                    name: self.collection.clone(),
                    schema: buffa::MessageField::some(schema.to_struct()?),
                    ..Default::default()
                },
                opts(),
            )
            .await;
        ignore_existing(created.map(|_| ()))?;
        Ok(())
    }

    pub async fn ready(&self) -> Result<(), PlatformError> {
        self.client
            .collections()
            .get(
                GetCollectionRequest {
                    namespace: self.namespace.clone(),
                    collection: self.collection.clone(),
                    ..Default::default()
                },
                opts(),
            )
            .await?;
        Ok(())
    }

    pub async fn upsert(&self, id: u64, input: &NoteInput) -> Result<(), PlatformError> {
        let mut doc = Document::new(id).source(json!({
            "title": &input.title,
            "body": &input.body,
            "tags": &input.tags,
        }));
        if let Some(embedding) = &input.embedding {
            doc = doc.vector("embedding", embedding.clone());
        }
        let op = loams::handle::ops::upsert(&doc)?;
        self.client
            .documents()
            .write(
                WriteDocumentsRequest {
                    namespace: self.namespace.clone(),
                    collection: self.collection.clone(),
                    ops: vec![op],
                    ..Default::default()
                },
                opts(),
            )
            .await?;
        Ok(())
    }

    pub async fn get(&self, id: u64) -> Result<Option<Value>, PlatformError> {
        let response = self
            .client
            .documents()
            .get(
                GetDocumentsRequest {
                    namespace: self.namespace.clone(),
                    collection: self.collection.clone(),
                    ids: vec![doc_id(id)],
                    ..Default::default()
                },
                opts(),
            )
            .await?;
        match response.documents.into_iter().next() {
            Some(doc) if doc.source.is_set() => Ok(Some(serde_json::to_value(&doc)?)),
            _ => Ok(None),
        }
    }

    pub async fn delete(&self, id: u64) -> Result<(), PlatformError> {
        let op = loams::handle::ops::delete(&DocId::from(id))?;
        self.client
            .documents()
            .write(
                WriteDocumentsRequest {
                    namespace: self.namespace.clone(),
                    collection: self.collection.clone(),
                    ops: vec![op],
                    ..Default::default()
                },
                opts(),
            )
            .await?;
        Ok(())
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<Value>, PlatformError> {
        let mut builder = Search::on(&self.collection).namespace(&self.namespace);
        if !query.q.trim().is_empty() {
            builder = builder.text(TextRetriever::fields(&query.q, ["title", "body"]));
        }
        if let Some(tag) = &query.tag {
            builder = builder.filter(Filter::eq("tags", tag.as_str()));
        }
        let request = builder.limit(query.limit).build()?;
        let response = self.client.search().query(request, opts()).await?;
        response
            .hits
            .into_iter()
            .map(|hit| serde_json::to_value(&hit).map_err(PlatformError::from))
            .collect()
    }
}

fn opts() -> CallOptionsOverrides {
    CallOptionsOverrides::new()
}

fn doc_id(id: u64) -> DocumentId {
    DocumentId {
        id: Some(document_id::Id::Uint(id)),
        ..Default::default()
    }
}

fn ignore_existing(result: Result<(), LoamsError>) -> Result<(), LoamsError> {
    match result {
        Err(error) if error.kind == ErrorKind::AlreadyExists => Ok(()),
        other => other,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error(transparent)]
    Loams(#[from] LoamsError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl PlatformError {
    pub fn kind(&self) -> Option<ErrorKind> {
        match self {
            Self::Loams(error) => Some(error.kind),
            Self::Json(_) => None,
        }
    }
}
