use schema::*;

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
