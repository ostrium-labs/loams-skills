use domain::{NoteInput, SearchQuery};
use platform::{Config, Platform};

#[tokio::test]
#[ignore = "needs a running `loams dev` and LOAMS_ENDPOINT"]
async fn note_roundtrip() {
    let config = Config::from_env().expect("config");
    let platform = Platform::connect(&config).await.expect("connect");
    let id = 424_242_u64;
    let input = NoteInput {
        id: Some(id),
        title: "Refund policy".into(),
        body: "the refund window is thirty days".into(),
        tags: vec!["policy".into()],
        embedding: None,
    };
    input.validate().expect("valid");
    platform.upsert(id, &input).await.expect("upsert");
    let note = platform.get(id).await.expect("get");
    assert!(note.is_some());
    let hits = platform
        .search(&SearchQuery {
            q: "refund".into(),
            tag: Some("policy".into()),
            limit: 10,
        })
        .await
        .expect("search");
    assert!(!hits.is_empty());
    platform.delete(id).await.expect("delete");
    assert!(platform.get(id).await.expect("get after delete").is_none());
}
