use serde::{Deserialize, Serialize};

pub mod methods;

#[derive(Serialize, Deserialize)]
pub struct File {
    pub hkey: String,
    pub name: String,
    pub size: u64,
}
