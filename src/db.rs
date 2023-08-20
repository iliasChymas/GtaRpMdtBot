use dotenv::dotenv;
use mongodb::{bson::doc, options::ClientOptions, Collection};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Criminal {
    name: String,
    felonies: Vec<String>,
    id_number: String,
}

impl ToString for Criminal {
    fn to_string(&self) -> String {
        let felonies: &str = &self.felonies.join(", ");
        return format!(
            "NAME: {}\nID NUMBER: {}\nFELONIES: {}",
            self.name, self.id_number, felonies
        );
    }
}

impl Criminal {
    pub fn new(name: &str, id_number: &str) -> Self {
        Self {
            name: String::from(name),
            id_number: String::from(id_number),
            felonies: vec![],
        }
    }
}

pub struct Database {
    collection: Collection<Criminal>,
}
impl serenity::prelude::TypeMapKey for Database {
    type Value = Database;
}
async fn connect_to_mongo() -> mongodb::Collection<Criminal> {
    dotenv().ok();
    // Parse a connection string into an options struct.
    let client_options =
        ClientOptions::parse(std::env::var("DB_URI").expect("No db_uri specified"))
            .await
            .unwrap();

    // Get a handle to the deployment.
    let client = mongodb::Client::with_options(client_options).unwrap();

    // List the names of the databases in that deployment.
    let criminal_collection = client.database("Mdt").collection::<Criminal>("Criminals");
    return criminal_collection;
}

impl Database {
    pub async fn new() -> Self {
        let con = connect_to_mongo().await;
        Self { collection: con }
    }

    pub async fn insert_criminal(&self, criminal: &Criminal) -> Result<(), ()> {
        self.collection.insert_one(criminal, None).await.unwrap();
        Ok(())
    }

    pub async fn add_felony(&self, id: &str, felonies: &Vec<&str>) -> Result<(), ()> {
        let filter = doc! {"id_number": id};
        let update = doc! {"$push" : {"felonies": {"$each" : felonies}}};
        self.collection
            .update_one(filter, update, None)
            .await
            .expect("Update failed"); // TODO handle error
        Ok(())
    }

    pub async fn get_criminal_by_id(&self, id: &str) -> Option<Criminal> {
        let filter = doc! { "id_number" : id};
        self.collection.find_one(filter, None).await.unwrap()
    }
}
