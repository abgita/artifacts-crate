use std::fmt::Debug;
use std::path::{Path};
use std::time::Duration;
use once_cell::sync::{OnceCell};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::{RwLock, RwLockReadGuard};

/*
The artifacts module provides a generic Artifact<T> struct for managing shared read/write access to data stored in a JSON file.
The Artifact struct supports initializing the data from a JSON file, getting a read lock on the data, updating the data, and automatically reloading of the data at a specified interval.

The Artifact struct is designed to be used with a static global variable and ensures safe concurrent access to the data across multiple threads.
The ARTIFACTS_PATH environment variable is used to locate the JSON file and defaults to "./artifacts" if not found.

Usage example:

1. Define your struct and make sure it implements DeserializeOwned:

#[derive(Deserialize, Debug)]
struct ExampleStruct {
    field1: String,
    field2: u32,
}

2. Initialize the Artifact with your struct:

pub static EXAMPLE_LIST: Artifact<ExampleStruct> = Artifact::new();

3. In your main function or another appropriate place, initialize the Artifact data and spawn the watch task:

#[tokio::main]
async fn main() {
    // Initialize the EXAMPLE_LIST data
    EXAMPLE_LIST.init("example.json").await.unwrap();

    // Spawn a background task to watch and reload the EXAMPLE_LIST data every 6 hours
    tokio::spawn(EXAMPLE_LIST.watch("example.json".to_string(), 6 * 60 * 60));

    {
        // Example of getting the data
        let example_data = EXAMPLE_LIST.get().await.unwrap();
        println!("Example data: {:?}", *example_data);
    }

    {
        // Example of updating the data (assuming you have new_data available)
        let new_data = ExampleStruct { field1: "New field1 value".to_string(), field2: 42 };
        EXAMPLE_LIST.update(new_data).await.unwrap();
    }
}
*/

// The Artifact struct wraps an object of type T, which will be loaded from a JSON file.
// It uses OnceCell to ensure that the data is initialized only once, and RwLock to allow
// concurrent reads while providing exclusive access for updates.
pub struct Artifact<T> {
    data: OnceCell<RwLock<T>>,
}

impl<T: Debug + serde::de::DeserializeOwned + Send + Sync + 'static> Artifact<T> {
    // Creates a new, uninitialized Artifact instance.
    pub const fn new() -> Self {
        Artifact {
            data: OnceCell::new(),
        }
    }

    // Initializes the Artifact data by loading it from the specified JSON file.
    // This method can be called multiple times, but the data will be initialized only once.
    // If the data is already initialized, this method does nothing.
    pub async fn init(&self, artifact_file: &str) -> Result<(), ArtifactError> {
        if self.data.get().is_none() {
            let artifacts_path = get_env_or_default("ARTIFACTS_PATH", "artifacts".to_string());
            let path = Path::new(&artifacts_path).join(artifact_file);

            let data = get_data::<T>(&path).await.map_err(|err| ArtifactError::InitializationError(err.to_string()))?;

            self.data.set(RwLock::new(data)).unwrap();
        }
        Ok(())
    }

    // Provides read access to the Artifact data.
    // This method returns a read guard, which allows multiple concurrent reads.
    pub async fn get(&self) -> Result<RwLockReadGuard<'_, T>, ArtifactError> {
        let data_lock = self.data.get().expect("Artifact is not initialized");
        Ok(data_lock.read().await)
    }

    // Updates the Artifact data with the provided new_data.
    // This method provides exclusive write access to the data, blocking other reads and writes
    // while the update is in progress.
    pub async fn update(&self, new_data: T) -> Result<(), ArtifactError> {
        let data_lock = self.data.get().expect("Artifact is not initialized");
        let mut data = data_lock.write().await;
        *data = new_data;

        Ok(())
    }

    // Starts a task that periodically reloads the Artifact data from the specified JSON file.
    // The task runs indefinitely, reloading the data at the specified interval in seconds.
    pub async fn watch(&self, artifact_file: String, interval_secs: u64) -> Result<(), ArtifactError> {
        let artifacts_path = get_env_or_default("ARTIFACTS_PATH", "artifacts".to_string());
        let path = Path::new(&artifacts_path).join(artifact_file);

        loop {
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            match get_data::<T>(&path).await {
                Ok(new_data) => {
                    if let Err(e) = self.update(new_data).await {
                        return Err(ArtifactError::UpdateError(e.to_string()));
                    }
                }
                Err(e) => {
                    return Err(ArtifactError::WatchError(e.to_string()));
                }
            }
        }
    }
}

// A helper function to read and deserialize data from a JSON file.
// The function is generic over the type T, which must implement the DeserializeOwned trait.
async fn get_data<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ArtifactError> {
    let mut file = File::open(path).await.map_err(|err| ArtifactError::IoError(err))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await.map_err(|err| ArtifactError::IoError(err))?;
    let data: T = serde_json::from_str(&contents).map_err(|err| ArtifactError::SerdeError(err))?;
    Ok(data)
}

pub fn get_env_or_default(key: &str, default: String) -> String {
    std::env::var(key).unwrap_or(default)
}

#[derive(Debug)]
pub enum ArtifactError {
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    InitializationError(String),
    UpdateError(String),
    WatchError(String),
}

impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactError::IoError(err) => write!(f, "I/O error: {}", err),
            ArtifactError::SerdeError(err) => write!(f, "JSON serialization/deserialization error: {}", err),
            ArtifactError::InitializationError(msg) => write!(f, "Initialization error: {}", msg),
            ArtifactError::UpdateError(msg) => write!(f, "Update error: {}", msg),
            ArtifactError::WatchError(msg) => write!(f, "Watch error: {}", msg),
        }
    }
}

impl std::error::Error for ArtifactError {}

impl From<std::io::Error> for ArtifactError {
    fn from(err: std::io::Error) -> Self {
        ArtifactError::IoError(err)
    }
}

impl From<serde_json::Error> for ArtifactError {
    fn from(err: serde_json::Error) -> Self {
        ArtifactError::SerdeError(err)
    }
}
