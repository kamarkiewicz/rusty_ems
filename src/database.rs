use errors::*;
// use models::*;

use diesel;
use diesel::prelude::*;
pub use diesel::pg::PgConnection;
use super::api::Timestamp;

pub fn establish_connection(login: String, password: String, baza: String) -> Result<PgConnection> {
    let database_url = format!("postgres://{}:{}@localhost/{}", login, password, baza);
    PgConnection::establish(&database_url)
        .chain_err(|| format!("Error connecting to {}", database_url))
}

pub fn create_organizer_account(conn: &PgConnection,
                                newlogin: String,
                                newpassword: String)
                                -> Result<()> {
    use schema::person;
    use models::{Person, NewPerson};

    let organizer = NewPerson {
        login: newlogin.as_ref(),
        password: newpassword.as_ref(),
        is_organizer: true,
    };

    diesel::insert(&organizer)
        .into(person::table)
        .get_result::<Person>(conn)
        .chain_err(|| "unable to add organizer person to database")?;

    Ok(())
}

#[allow(unused_variables)]
#[allow(unused_imports)]
pub fn create_event(conn: &PgConnection,
                    login: String,
                    password: String,
                    eventname: String,
                    start_timestamp: Timestamp,
                    end_timestamp: Timestamp)
                    -> Result<()> {
    use schema::person;
    use models::{Person, Event, NewEvent};

    // TODO: authorize person as organizer
    // TODO: insert new event

    Ok(())
}
