use schema::*;
use api::Timestamp;

#[derive(Queryable)]
pub struct Person {
    pub id: i32,
    pub login: String,
    pub password: String,
    pub is_organizer: bool,
}

#[derive(Insertable)]
#[table_name="person"]
pub struct NewPerson<'a> {
    pub login: &'a str,
    pub password: &'a str,
    pub is_organizer: bool,
}

#[derive(Queryable)]
pub struct Event {
    pub id: i32,
    pub eventname: String,
    pub start_timestamp: Timestamp,
    pub end_timestmp: Timestamp,
}

#[derive(Insertable)]
#[table_name="event"]
pub struct NewEvent<'a> {
    pub eventname: &'a str,
    pub start_timestamp: Timestamp,
    pub end_timestamp: Timestamp,
}
