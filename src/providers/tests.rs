use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;

use super::{BoardInfo, Provider};
use crate::model::work_item::WorkItem;

/// A mock provider that tracks move_to_done and move_to_in_progress calls for testing.
struct MockProvider {
    provider_name: String,
    done_ids: Arc<Mutex<Vec<String>>>,
    in_progress_ids: Arc<Mutex<Vec<String>>>,
    created_items: Arc<Mutex<Vec<(String, Option<String>)>>>,
    should_fail: bool,
    supports_create: bool,
}

impl MockProvider {
    fn new(name: &str) -> Self {
        Self {
            provider_name: name.to_string(),
            done_ids: Arc::new(Mutex::new(Vec::new())),
            in_progress_ids: Arc::new(Mutex::new(Vec::new())),
            created_items: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
            supports_create: false,
        }
    }

    fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    fn with_create_support(mut self) -> Self {
        self.supports_create = true;
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

    async fn move_to_in_progress(&self, source_id: &str) -> Result<()> {
        if self.should_fail {
            anyhow::bail!("Mock failure");
        }
        self.in_progress_ids
            .lock()
            .unwrap()
            .push(source_id.to_string());
        Ok(())
    }

    async fn create_item(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> Result<Option<WorkItem>> {
        if !self.supports_create {
            return Ok(None);
        }
        if self.should_fail {
            anyhow::bail!("Mock create failure");
        }
        self.created_items
            .lock()
            .unwrap()
            .push((title.to_string(), description.map(String::from)));

        Ok(Some(WorkItem {
            id: format!("MOCK-1"),
            source_id: Some("mock-source-id".to_string()),
            title: title.to_string(),
            description: description.map(String::from),
            status: Some("Todo".to_string()),
            priority: None,
            labels: Vec::new(),
            source: self.provider_name.clone(),
            team: None,
            url: Some("https://mock.test/item/1".to_string()),
        }))
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

#[tokio::test]
async fn move_to_in_progress_calls_correct_provider() {
    let provider = MockProvider::new("Trello");
    let in_progress_ids = provider.in_progress_ids.clone();

    provider.move_to_in_progress("card-123").await.unwrap();

    assert_eq!(
        in_progress_ids.lock().unwrap().as_slice(),
        &["card-123"]
    );
}

#[tokio::test]
async fn move_to_in_progress_default_is_noop() {
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
    }

    let provider = NoopProvider;
    assert!(provider.move_to_in_progress("anything").await.is_ok());
}

#[tokio::test]
async fn move_to_in_progress_propagates_errors() {
    let provider = MockProvider::new("Trello").with_failure();
    let result = provider.move_to_in_progress("card-123").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mock failure"));
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

// --- create_item tests ---

#[tokio::test]
async fn create_item_default_returns_none() {
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
    }

    let provider = NoopProvider;
    let result = provider.create_item("Test task", None).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn create_item_with_mock_provider() {
    let provider = MockProvider::new("TestProvider").with_create_support();
    let created = provider.created_items.clone();

    let result = provider
        .create_item("New feature", Some("Build it fast"))
        .await
        .unwrap();

    assert!(result.is_some());
    let item = result.unwrap();
    assert_eq!(item.title, "New feature");
    assert_eq!(item.description, Some("Build it fast".to_string()));
    assert_eq!(item.source, "TestProvider");
    assert!(item.url.is_some());

    let items = created.lock().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, "New feature");
    assert_eq!(items[0].1, Some("Build it fast".to_string()));
}

#[tokio::test]
async fn create_item_without_description() {
    let provider = MockProvider::new("TestProvider").with_create_support();
    let created = provider.created_items.clone();

    let result = provider.create_item("Simple task", None).await.unwrap();
    assert!(result.is_some());

    let items = created.lock().unwrap();
    assert_eq!(items[0].1, None);
}

#[tokio::test]
async fn create_item_propagates_errors() {
    let provider = MockProvider::new("FailProvider")
        .with_create_support()
        .with_failure();

    let result = provider.create_item("Will fail", None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mock create failure"));
}

#[tokio::test]
async fn create_item_unsupported_provider_returns_none() {
    // Provider without create support should return None, not error
    let provider = MockProvider::new("NoCreate");
    let result = provider.create_item("Test", None).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn create_item_tries_providers_in_order() {
    // Simulate the app logic: try each provider until one works
    let providers: Vec<Box<dyn Provider>> = vec![
        Box::new(MockProvider::new("NoCreate1")),     // Doesn't support create
        Box::new(MockProvider::new("NoCreate2")),     // Doesn't support create
        Box::new(MockProvider::new("Creator").with_create_support()), // Supports create
    ];

    let mut created = false;
    for provider in &providers {
        match provider.create_item("Test task", None).await {
            Ok(Some(item)) => {
                assert_eq!(item.source, "Creator");
                created = true;
                break;
            }
            Ok(None) => continue,
            Err(_) => break,
        }
    }

    assert!(created, "Should have created an item via the Creator provider");
}

#[tokio::test]
async fn create_item_skips_failing_provider() {
    let providers: Vec<Box<dyn Provider>> = vec![
        Box::new(MockProvider::new("Broken").with_create_support().with_failure()),
        Box::new(MockProvider::new("Working").with_create_support()),
    ];

    let mut result_item = None;
    for provider in &providers {
        match provider.create_item("Test", None).await {
            Ok(Some(item)) => {
                result_item = Some(item);
                break;
            }
            Ok(None) => continue,
            Err(_) => continue, // Skip broken provider
        }
    }

    let item = result_item.expect("Should have fallen through to Working provider");
    assert_eq!(item.source, "Working");
}

#[test]
fn create_item_result_has_correct_fields() {
    // Verify WorkItem structure for a created item
    let item = WorkItem {
        id: "TRE-123".to_string(),
        source_id: Some("full-trello-id".to_string()),
        title: "My new task".to_string(),
        description: Some("Detailed description".to_string()),
        status: Some("Todo".to_string()),
        priority: None,
        labels: vec!["feature".to_string()],
        source: "Trello".to_string(),
        team: Some("My Board".to_string()),
        url: Some("https://trello.com/c/abc123".to_string()),
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: WorkItem = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "TRE-123");
    assert_eq!(deserialized.source_id, Some("full-trello-id".to_string()));
    assert_eq!(deserialized.title, "My new task");
    assert_eq!(
        deserialized.description,
        Some("Detailed description".to_string())
    );
    assert_eq!(deserialized.status, Some("Todo".to_string()));
    assert_eq!(deserialized.labels, vec!["feature"]);
    assert_eq!(deserialized.source, "Trello");
    assert_eq!(deserialized.url, Some("https://trello.com/c/abc123".to_string()));
}
