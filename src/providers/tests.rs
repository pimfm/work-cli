use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;

use super::{BoardInfo, Provider};
use crate::model::work_item::WorkItem;

/// A mock provider that tracks move_to_done calls for testing.
struct MockProvider {
    provider_name: String,
    done_ids: Arc<Mutex<Vec<String>>>,
    should_fail: bool,
}

impl MockProvider {
    fn new(name: &str) -> Self {
        Self {
            provider_name: name.to_string(),
            done_ids: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl Provider for MockProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }

    async fn fetch_items(&self) -> Result<Vec<WorkItem>> {
        Ok(vec![])
    }

    async fn list_boards(&self) -> Result<Vec<BoardInfo>> {
        Ok(vec![])
    }

    async fn move_to_done(&self, source_id: &str) -> Result<()> {
        if self.should_fail {
            anyhow::bail!("Mock failure");
        }
        self.done_ids.lock().unwrap().push(source_id.to_string());
        Ok(())
    }
}

fn make_work_item(id: &str, source: &str, source_id: Option<&str>) -> WorkItem {
    WorkItem {
        id: id.to_string(),
        source_id: source_id.map(|s| s.to_string()),
        title: format!("Test item {id}"),
        description: None,
        status: Some("Todo".into()),
        priority: None,
        labels: vec![],
        source: source.to_string(),
        team: None,
        url: None,
    }
}

#[test]
fn work_item_has_source_id() {
    let item = make_work_item("abc123", "Trello", Some("full-trello-card-id-24chars"));
    assert_eq!(item.source_id, Some("full-trello-card-id-24chars".to_string()));
    assert_eq!(item.id, "abc123");
}

#[test]
fn work_item_source_id_optional() {
    let item = make_work_item("abc123", "Trello", None);
    assert_eq!(item.source_id, None);
}

#[tokio::test]
async fn move_to_done_calls_correct_provider() {
    let provider = MockProvider::new("Trello");
    let done_ids = provider.done_ids.clone();

    provider.move_to_done("card-123").await.unwrap();

    assert_eq!(done_ids.lock().unwrap().as_slice(), &["card-123"]);
}

#[tokio::test]
async fn move_to_done_default_is_noop() {
    // Test that the default trait implementation doesn't error
    struct NoopProvider;

    #[async_trait]
    impl Provider for NoopProvider {
        fn name(&self) -> &str {
            "Noop"
        }
        async fn fetch_items(&self) -> Result<Vec<WorkItem>> {
            Ok(vec![])
        }
        async fn list_boards(&self) -> Result<Vec<BoardInfo>> {
            Ok(vec![])
        }
        // move_to_done intentionally not implemented — uses default
    }

    let provider = NoopProvider;
    assert!(provider.move_to_done("anything").await.is_ok());
}

#[tokio::test]
async fn move_to_done_propagates_errors() {
    let provider = MockProvider::new("Trello").with_failure();
    let result = provider.move_to_done("card-123").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mock failure"));
}

#[tokio::test]
async fn find_provider_by_source_name() {
    let providers: Vec<Box<dyn Provider>> = vec![
        Box::new(MockProvider::new("Trello")),
        Box::new(MockProvider::new("Linear")),
        Box::new(MockProvider::new("Jira")),
    ];

    let item = make_work_item("ENG-1", "Linear", Some("uuid-123"));
    let source_id = item.source_id.as_ref().unwrap();

    // Find the matching provider and call move_to_done
    let matched = providers.iter().find(|p| p.name() == item.source);
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().name(), "Linear");
    assert!(matched.unwrap().move_to_done(source_id).await.is_ok());
}

#[test]
fn work_item_serialization_with_source_id() {
    let item = make_work_item("abc", "Trello", Some("full-id-here"));
    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("source_id"));
    assert!(json.contains("full-id-here"));

    let deserialized: WorkItem = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.source_id, Some("full-id-here".to_string()));
}

#[test]
fn work_item_serialization_without_source_id() {
    let item = make_work_item("abc", "Trello", None);
    let json = serde_json::to_string(&item).unwrap();
    // source_id should be omitted when None (skip_serializing_if)
    assert!(!json.contains("source_id"));

    // But deserialization should handle missing field
    let json_no_source_id = r#"{"id":"abc","title":"Test","labels":[],"source":"Trello"}"#;
    let deserialized: WorkItem = serde_json::from_str(json_no_source_id).unwrap();
    assert_eq!(deserialized.source_id, None);
}
