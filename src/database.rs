use errors::*;
use super::api::{Date, DateTime};

use diesel;
use diesel::prelude::*;
pub use diesel::pg::PgConnection;

use models::Person;

pub fn establish_connection(login: String, password: String, baza: String) -> Result<PgConnection> {
    let database_url = format!("postgres://{}:{}@localhost/{}", login, password, baza);
    PgConnection::establish(&database_url)
        .chain_err(|| format!("Error connecting to {}", database_url))
}

pub fn create_organizer_account(conn: &PgConnection,
                                newlogin: String,
                                newpassword: String)
                                -> Result<()> {
    use schema::persons;
    use models::{Person, NewPerson};

    let organizer = NewPerson {
        login: newlogin.as_ref(),
        password: newpassword.as_ref(),
        is_organizer: true,
    };

    let query = diesel::insert(&organizer).into(persons::table);
    // eprintln!("{}", debug_sql!(query));
    query
        .get_result::<Person>(conn)
        .chain_err(|| "unable to add organizer person to database")?;

    Ok(())
}

pub fn create_event(conn: &PgConnection,
                    login: String,
                    password: String,
                    eventname: String,
                    start_timestamp: DateTime,
                    end_timestamp: DateTime)
                    -> Result<()> {
    use schema::events;
    use models::{Event, NewEvent};

    // authorize person as organizer
    let person = authorize_person(&conn, login, password)?;
    must_have_organizer_rights(person)?;

    // insert new event
    let event = NewEvent {
        eventname: eventname.as_ref(),
        start_timestamp: start_timestamp,
        end_timestamp: end_timestamp,
    };

    /// INSERT INTO `events` (`eventname`, `start_timestamp`, `end_timestamp`) VALUES (?, ?, ?)
    let query = diesel::insert(&event).into(events::table);
    // eprintln!("{}", debug_sql!(query));
    query
        .get_result::<Event>(conn)
        .chain_err(|| "unable to add event to database")?;

    Ok(())
}

pub fn create_user(conn: &PgConnection,
                   login: String,
                   password: String,
                   newlogin: String,
                   newpassword: String)
                   -> Result<()> {
    use schema::persons;
    use models::{Person, NewPerson};

    let _ = authorize_person(&conn, login, password)?;

    let new_person = NewPerson {
        login: newlogin.as_ref(),
        password: newpassword.as_ref(),
        is_organizer: false,
    };

    let query = diesel::insert(&new_person).into(persons::table);
    // eprintln!("{}", debug_sql!(query));
    query
        .get_result::<Person>(conn)
        .chain_err(|| "unable to add regular person to database")?;

    Ok(())
}

fn authorize_person(conn: &PgConnection, login: String, password: String) -> Result<Person> {
    use schema::persons;

    /// SELECT `persons`.`id`, `persons`.`login`, `persons`.`password`, `persons`.`is_organizer`
    /// FROM `persons` WHERE `persons`.`login` = ? AND `persons`.`password` = ? LIMIT ?
    let query = persons::table
        .filter(persons::login.eq(login))
        .filter(persons::password.eq(password))
        .limit(1);
    // eprintln!("{}", debug_sql!(query));
    let authorized_person = query
        .first::<Person>(conn)
        .chain_err(|| "Error loading person")?;
    Ok(authorized_person)
}

fn must_have_organizer_rights(person: Person) -> Result<()> {
    if person.is_organizer {
        Ok(())
    } else {
        Err("Person doesn't have organizer rights".into())
    }
}
