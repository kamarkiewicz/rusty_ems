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

// #[derive(Queryable, Identifiable)]
// pub struct Talk {
//     pub id: i32,
//     pub title: String,
//     pub status: i16,
//     pub person_id: i32, // speakerlogin
//     pub event_id: Option<i32>, // eventname
//     pub start_timestamp: DateTime,
//     pub room: String,
//     pub initial_evaluation: i16,
//     pub eventname: String,
// }

// #[derive(Insertable)]
// #[table_name="talks"]
// pub struct NewTalk<'a> {
//     pub Talkname: &'a str,
//     pub start_timestamp: DateTime,
// }
