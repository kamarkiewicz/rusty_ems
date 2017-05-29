use schema::*;

#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub login: String,
    pub password: String,
    pub is_organizer: bool,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub login: &'a str,
    pub password: &'a str,
    pub is_organizer: bool,
}
