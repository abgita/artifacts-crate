use std::time::Duration;

use serde::{Deserialize, Serialize};

use artifacts_crate::Artifact;

#[derive(Deserialize, Debug)]
pub struct ExampleStruct {
    field1: String,
    field2: u32,
}

pub static EXAMPLE_LIST: Artifact<ExampleStruct> = Artifact::new();

#[tokio::main]
async fn main() {
    // we need to create the artifacts directory and the example.json file before we start the program
    EXAMPLE_LIST.init("example.json").await.unwrap();

    tokio::spawn(EXAMPLE_LIST.watch("example.json".to_string(), 2));

    {
        let example_data = EXAMPLE_LIST.get().await.unwrap();
        println!("initial Example data: {:?}", *example_data);
    }

    {
        let new_data = ExampleStruct { field1: "New field1 value".to_string(), field2: 42 };
        EXAMPLE_LIST.update(new_data).await.unwrap();
        let example_data = EXAMPLE_LIST.get().await.unwrap();
        println!("updated Example data: {:?}", *example_data);
    }


    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}


