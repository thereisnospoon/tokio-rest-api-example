use deadpool_postgres::tokio_postgres::Row;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub age: i32,
    pub sex: Sex
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Sex {
    Male,
    Female
}

impl From<&Sex> for &str {
    fn from(val: &Sex) -> Self {
        match val {
            Sex::Male => "m",
            Sex::Female => "f"
        }
    }
}

impl From<&str> for Sex {
    fn from(s: &str) -> Self {
        match s {
            "m" => Sex::Male,
            "f" => Sex::Female,
            _ => unreachable!("Unknown sex")
        }
    }
}

impl From<Row> for User {
    fn from(row: Row) -> Self {
        let sex: &str = row.get("sex");
        User {
            id: row.get("id"),
            name: row.get("name"),
            age: row.get("age"),
            sex: sex.into()
        }
    }
}