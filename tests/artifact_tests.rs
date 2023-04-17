use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use tokio::sync::RwLock;

use artifacts_crate::Artifact;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExampleStruct {
    field1: String,
    field2: u32,
}

#[tokio::test]
async fn test_init() {
    let tmp_dir = tempdir().unwrap();
    let artifact_file_path = tmp_dir.path().join("example.json");
    std::fs::write(
        &artifact_file_path,
        r#"{"field1": "Test string", "field2": 123}"#,
    )
        .unwrap();

    let example_list = Arc::new(Artifact::<ExampleStruct>::new());
    example_list.init(artifact_file_path.to_str().unwrap()).await.unwrap();
    let data = example_list.get().await.unwrap();
    assert_eq!(
        *data,
        ExampleStruct {
            field1: "Test string".to_string(),
            field2: 123,
        }
    );
}

#[tokio::test]
async fn test_update() {
    let tmp_dir = tempdir().unwrap();
    let artifact_file_path = tmp_dir.path().join("example.json");
    std::fs::write(
        &artifact_file_path,
        r#"{"field1": "Test string", "field2": 123}"#,
    )
        .unwrap();

    let example_list = Arc::new(Artifact::<ExampleStruct>::new());
    example_list.init(artifact_file_path.to_str().unwrap()).await.unwrap();

    let new_data = ExampleStruct {
        field1: "Updated string".to_string(),
        field2: 456,
    };
    example_list.update(new_data.clone()).await.unwrap();
    let data = example_list.get().await.unwrap();
    assert_eq!(*data, new_data);
}

pub static EXAMPLE_LIST: Artifact<ExampleStruct> = Artifact::new();

#[tokio::test]
async fn test_concurrent_read_and_update() {
    let tmp_dir = tempdir().unwrap();
    let artifact_file_path = tmp_dir.path().join("example_concurrent.json");
    std::fs::write(
        &artifact_file_path,
        r#"{"field1": "Initial string", "field2": 1}"#,
    )
        .unwrap();

    EXAMPLE_LIST.init(artifact_file_path.to_str().unwrap()).await.unwrap();

    let num_tasks = 100;
    let mut tasks = Vec::with_capacity(num_tasks);
    let updates: Arc<RwLock<Vec<ExampleStruct>>> = Arc::new(RwLock::new(Vec::new()));

    for i in 0..num_tasks {
        let updates_clone = Arc::clone(&updates);

        tasks.push(tokio::spawn(async move {
            if i % 2 == 0 {
                // Read task
                let data = EXAMPLE_LIST.get().await.unwrap();
                assert!(updates_clone.read().await.contains(&data));
            } else {
                // Update task
                let new_data = ExampleStruct {
                    field1: format!("Updated string {}", i),
                    field2: i as u32,
                };
                EXAMPLE_LIST.update(new_data.clone()).await.unwrap();
                {
                    let mut updates = updates_clone.write().await;
                    updates.push(new_data);
                }
            }
        }));
    }

    futures::future::join_all(tasks).await;
}

#[tokio::test]
async fn test_init_missing_file() {
    let tmp_dir = tempdir().unwrap();
    let artifact_file_path = tmp_dir.path().join("missing.json");

    let example_list = Arc::new(Artifact::<ExampleStruct>::new());
    let result = example_list.init(artifact_file_path.to_str().unwrap()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_init_invalid_json() {
    let tmp_dir = tempdir().unwrap();
    let artifact_file_path = tmp_dir.path().join("invalid.json");
    std::fs::write(&artifact_file_path, r#"{"field1": "Test string", "field2": "invalid"}"#).unwrap();

    let example_list = Arc::new(Artifact::<ExampleStruct>::new());
    let result = example_list.init(artifact_file_path.to_str().unwrap()).await;

    assert!(result.is_err());
}

mod test {
    use super::*;

    pub static EXAMPLE_LIST_2: Artifact<ExampleStruct> = Artifact::new();

    #[tokio::test]
    async fn test_watch() {
        let tmp_dir = tempdir().unwrap();
        let artifact_file_path = tmp_dir.path().join("example_watch.json");
        std::fs::write(
            &artifact_file_path,
            r#"{"field1": "Test string", "field2": 123}"#,
        )
            .unwrap();

        EXAMPLE_LIST_2.init(artifact_file_path.to_str().unwrap()).await.unwrap();

        let new_data = ExampleStruct {
            field1: "Updated string".to_string(),
            field2: 456,
        };
        std::fs::write(&artifact_file_path, serde_json::to_string(&new_data).unwrap()).unwrap();

        let watch_handle = tokio::spawn(EXAMPLE_LIST_2.watch(artifact_file_path.to_str().unwrap().to_string(), 1));

        // Add a delay to give the watch task enough time to pick up the changes
        tokio::time::sleep(Duration::from_secs(2)).await;

        {
            let data = EXAMPLE_LIST_2.get().await.unwrap();
            assert_eq!(*data, new_data);
        }

        watch_handle.abort();
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct ComplexStruct {
    field1: String,
    field2: u32,
    nested: Vec<ExampleStruct>,
}

#[tokio::test]
async fn test_complex_struct() {
    let tmp_dir = tempdir().unwrap();
    let artifact_file_path = tmp_dir.path().join("complex_example.json");
    std::fs::write(
        &artifact_file_path,
        r#"{
            "field1": "Test string",
            "field2": 123,
            "nested": [
                {"field1": "Nested string 1", "field2": 1},
                {"field1": "Nested string 2", "field2": 2}
            ]
        }"#,
    )
        .unwrap();

    let example_list = Arc::new(Artifact::<ComplexStruct>::new());
    example_list.init(artifact_file_path.to_str().unwrap()).await.unwrap();

    let data = example_list.get().await.unwrap();
    assert_eq!(
        *data,
        ComplexStruct {
            field1: "Test string".to_string(),
            field2: 123,
            nested: vec![
                ExampleStruct {
                    field1: "Nested string 1".to_string(),
                    field2: 1,
                },
                ExampleStruct {
                    field1: "Nested string 2".to_string(),
                    field2: 2,
                },
            ],
        }
    );
}
