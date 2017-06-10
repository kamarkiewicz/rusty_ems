use schema::*;
use api::DateTime;

#[derive(Queryable, Identifiable)]
pub struct Person {
    pub id: i32,
    pub login: String,
    pub password: String,
    pub is_organizer: bool,
}

#[derive(Insertable)]
#[table_name="persons"]
pub struct NewPerson<'a> {
    pub login: &'a str,
    pub password: &'a str,
    pub is_organizer: bool,
}

#[derive(Queryable, Identifiable)]
pub struct Event {
    pub id: i32,
    pub eventname: String,
    pub start_timestamp: DateTime,
    pub end_timestmp: DateTime,
}

#[derive(Insertable)]
#[table_name="events"]
pub struct NewEvent<'a> {
    pub eventname: &'a str,
    pub start_timestamp: DateTime,
    pub end_timestamp: DateTime,
}
