use errors::*;
use super::api::{Date, DateTime};

use diesel;
use diesel::prelude::*;
pub use diesel::pg::PgConnection;

use models::Person;

/// (*) open <baza> <login> <password>
/// przekazuje dane umożliwiające podłączenie Twojego programu do bazy - nazwę bazy,
/// login oraz hasło, wywoływane dokładnie jeden raz, w pierwszej linii wejścia
/// zwraca status OK/ERROR w zależności od tego czy udało się nawiązać połączenie z bazą
pub fn establish_connection(login: String, password: String, baza: String) -> Result<PgConnection> {
    let database_url = format!("postgres://{}:{}@localhost/{}", login, password, baza);
    PgConnection::establish(&database_url)
        .chain_err(|| format!("Error connecting to {}", database_url))
}

/// (*) organizer <secret> <newlogin> <newpassword>
/// tworzy uczestnika <newlogin> z uprawnieniami organizatora i hasłem <newpassword>,
/// argument <secret> musi być równy d8578edf8458ce06fbc5bb76a58c5ca4 // zwraca status OK/ERROR
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

/// (*O) event <login> <password> <eventname> <start_timestamp> <end_timestamp>
/// rejestracja wydarzenia, napis <eventname> jest unikalny
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
    must_have_organizer_rights(&person)?;

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

/// (*O) user <login> <password> <newlogin> <newpassword>
/// rejestracja nowego uczestnika
/// <login> i <password> służą do autoryzacji wywołującego funkcję,
/// który musi posiadać uprawnienia organizatora,
/// <newlogin> <newpassword> są danymi nowego uczestnika,
/// <newlogin> jest unikalny
pub fn create_user(conn: &PgConnection,
                   login: String,
                   password: String,
                   newlogin: String,
                   newpassword: String)
                   -> Result<()> {
    use schema::persons;
    use models::{Person, NewPerson};

    // authorize person as organizer
    let person = authorize_person(&conn, login, password)?;
    must_have_organizer_rights(&person)?;

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

/// (*O) talk <login> <password>
///     <speakerlogin> <talk> <title> <start_timestamp> <room> <initial_evaluation> <eventname>
/// rejestracja referatu/zatwierdzenie referatu spontanicznego,
/// <talk> jest unikalnym identyfikatorem referatu,
/// <initial_evaluation> jest oceną organizatora w skali 0-10 – jest to ocena traktowana
///     tak samo jak ocena uczestnika obecnego na referacie,
/// <eventname> jest nazwą wydarzenia, którego częścią jest dany referat - może być pustym
///     napisem, co oznacza, że referat nie jest przydzielony do jakiegokolwiek wydarzenia
pub fn register_or_accept_talk(conn: &PgConnection,
                               login: String,
                               password: String,
                               speakerlogin: String,
                               talk: String,
                               title: String,
                               start_timestamp: DateTime,
                               room: String,
                               initial_evaluation: i16,
                               eventname: String)
                               -> Result<()> {
    Err("UNIMPL".into())
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

fn must_have_organizer_rights(person: &Person) -> Result<()> {
    if person.is_organizer {
        Ok(())
    } else {
        Err("Person doesn't have organizer rights".into())
    }
}
