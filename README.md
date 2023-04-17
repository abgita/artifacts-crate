# Artifacts Crate

A crate for managing shared read/write access to data stored in a JSON file. The `Artifact` struct supports initializing the data from a JSON file, getting a read lock on the data, updating the data, and watching for changes on the file to automatically reload the data at a specified interval.

## Usage

- Add the following to your `Cargo.toml` file:

```toml
[dependencies]
artifacts-crate = "0.1.0"
```

- Define your struct and make sure it implements DeserializeOwned:

```rust
#[derive(Deserialize, Debug)]
struct ExampleStruct {
    field1: String,
    field2: u32,
}
```

- Initialize the Artifact with your struct:

```rust
pub static EXAMPLE_LIST: Artifact<ExampleStruct> = Artifact::new();
```

- In your main function or another appropriate place, initialize the Artifact data and spawn the watch task:

```rust
#[tokio::main]
async fn main() {
    // Initialize the EXAMPLE_LIST data
    EXAMPLE_LIST.init("example.json").await.unwrap();

    // Spawn a background task to watch and reload the EXAMPLE_LIST data every 6 hours
    tokio::spawn(EXAMPLE_LIST.watch("example.json".to_string(), 6 * 60 * 60));

    // Example of getting the data
    {
        let example_data = EXAMPLE_LIST.get().await.unwrap();
        println!("Example data: {:?}", *example_data);
    }
    
    // Example of updating the data
    {
        let new_data = ExampleStruct { field1: "New field1 value".to_string(), field2: 42 };
        EXAMPLE_LIST.update(new_data).await.unwrap();
    }
}
```
For more details, please refer to the crate documentation or read the comments in the source code.

## License
This project is licensed under the MIT License.
