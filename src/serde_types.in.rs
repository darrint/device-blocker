
#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub mac: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Table {
    pub entries: Vec<Entry>,
}


