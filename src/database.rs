use errors::*;
use super::api::{Date, DateTime};

use diesel;
use diesel::prelude::*;
pub use diesel::pg::PgConnection;

use models::Person;
use diesel::expression::sql;

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

    // INSERT INTO `persons` (`login`, `password`, `is_organizer`) VALUES (?, ?, ?)
    let query = diesel::insert(&organizer).into(persons::table);
    eprintln!("{}", debug_sql!(query));
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
    // use schema::talks;
    // use models::{Talk, NewTalk};

    // authorize person as organizer
    let person = authorize_person(&conn, login, password)?;
    must_have_organizer_rights(&person)?;

    Err("UNIMPL".into())
}

/// (*U) register_user_for_event <login> <password> <eventname>
/// rejestracja uczestnika <login> na wydarzenie <eventname>
pub fn register_user_for_event(conn: &PgConnection,
                               login: String,
                               password: String,
                               eventname: String)
                               -> Result<()> {
    let person = authorize_person(&conn, login, password)?;

    use schema::events;
    use models::Event;
    let query = events::table
        .filter(events::eventname.eq(eventname))
        .limit(1);
    let event = query
        .first::<Event>(conn)
        .chain_err(|| "Error loading event")?;

    use schema::person_registered_for_event;
    use diesel::types::{Bool, Integer};
    let query = sql::<Bool>(
        r#"INSERT INTO person_registered_for_event(person_id, event_id) VALUES ($1, $2)"#)
        .bind::<Integer, _>(person.id)
        .bind::<Integer, _>(event.id);
    // eprintln!("{}", debug_sql!(query));
    query
        .execute(conn)
        .chain_err(|| "Person can't be registered for event")?;

    Ok(())
}

/// (*U) attendance <login> <password> <talk>
/// odnotowanie faktycznej obecności uczestnika <login> na referacie <talk>

/// (*U) evaluation <login> <password> <talk> <rating>
/// ocena referatu <talk> w skali 0-10 przez uczestnika <login>

/// (O) reject <login> <password> <talk>
/// usuwa referat spontaniczny <talk> z listy zaproponowanych,

/// (U) proposal  <login> <password> <talk> <title> <start_timestamp>
/// propozycja referatu spontanicznego, <talk> - unikalny identyfikator referatu

/// (U) friends <login1> <password> <login2>
/// uczestnik <login1> chce nawiązać znajomość z uczestnikiem <login2>, znajomość uznajemy
/// za nawiązaną jeśli obaj uczestnicy chcą ją nawiązać tj. po wywołaniach
/// friends <login1> <password1> <login2> i friends <login2> <password2> <login1>

/// (*N) user_plan <login> <limit>
/// zwraca plan najbliższych referatów z wydarzeń, na które dany uczestnik jest zapisany
/// (wg rejestracji na wydarzenia) posortowany wg czasu rozpoczęcia,
/// wypisuje pierwsze <limit> referatów, przy czym 0 oznacza, że należy wypisać wszystkie
/// Atrybuty zwracanych krotek: 
///   <login> <talk> <start_timestamp> <title> <room>

/// (*N) day_plan <timestamp>
/// zwraca listę wszystkich referatów zaplanowanych na dany dzień posortowaną rosnąco wg sal,
///     w drugiej kolejności wg czasu rozpoczęcia
///  <talk> <start_timestamp> <title> <room>

/// (*N) best_talks <start_timestamp> <end_timestamp> <limit> <all>
/// zwraca referaty rozpoczynające się w  danym przedziale czasowym posortowane malejąco
/// wg średniej oceny uczestników, przy czym jeśli <all> jest równe 1 należy wziąć
/// pod uwagę wszystkie oceny, w przeciwnym przypadku tylko oceny uczestników, którzy
/// byli na referacie obecni, wypisuje pierwsze <limit> referatów, przy czym 0 oznacza,
/// że należy wypisać wszystkie
///  <talk> <start_timestamp> <title> <room>

/// (*N) most_popular_talks <start_timestamp> <end_timestamp> <limit>
/// zwraca referaty rozpoczynające się w podanym przedziału czasowego posortowane malejąco
/// wg obecności, wypisuje pierwsze <limit> referatów, przy czym 0 oznacza,
/// że należy wypisać wszystkie
///  <talk> <start_timestamp> <title> <room>

/// (*U) attended_talks <login> <password>
/// zwraca dla danego uczestnika referaty, na których był obecny 
///  <talk> <start_timestamp> <title> <room>
fn attended_talks(conn: &PgConnection,
                  login: String,
                  password: String)
                  -> Result<()> {
    let person = authorize_person(&conn, login, password)?;

    use schema::{person_attended_for_talk, talks};
    use models::AttendedTalks;
    let query = sql::<AttendedTalks>(
        r#"SELECT talk, start_timestamp, title, room FROM person_attended_for_talk paft
           JOIN talks ON paft.talk_id=talks.id
           WHERE paft.person_id = 1 AND talks.start_timestamp >= 2;"#);
    let talks: Vec<AttendedTalks> = query.get_results(conn).chain_err(|| "sth goes wrong...")?;

    Err("".into())
}

/// (*O) abandoned_talks <login> <password>  <limit>
/// zwraca listę referatów posortowaną malejąco wg liczby uczestników
/// <number> zarejestrowanych na wydarzenie obejmujące referat,
/// którzy nie byli na tym referacie obecni, wypisuje pierwsze <limit> referatów,
/// przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <start_timestamp> <title> <room> <number>

/// (N) recently_added_talks <limit>
/// zwraca listę ostatnio zarejestrowanych referatów, wypisuje ostatnie <limit> referatów
/// wg daty zarejestrowania, przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <speakerlogin> <start_timestamp> <title> <room>

/// (U/O) rejected_talks <login> <password>
/// jeśli wywołujący ma uprawnienia organizatora zwraca listę wszystkich odrzuconych referatów
/// spontanicznych, w przeciwnym przypadku listę odrzuconych referatów wywołującego ją uczestnika 
///  <talk> <speakerlogin> <start_timestamp> <title>

/// (O) proposals <login> <password>
/// zwraca listę propozycji referatów spontanicznych do zatwierdzenia lub odrzucenia,
/// zatwierdzenie lub odrzucenie referatu polega na wywołaniu przez organizatora
/// funkcji talk lub reject z odpowiednimi parametrami
///  <talk> <speakerlogin> <start_timestamp> <title>

/// (U) friends_talks <login> <password> <start_timestamp> <end_timestamp> <limit>
/// lista referatów  rozpoczynających się w podanym przedziale czasowym wygłaszanych
/// przez znajomych danego uczestnika posortowana wg czasu rozpoczęcia,
/// wypisuje pierwsze <limit> referatów, przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <speakerlogin> <start_timestamp> <title> <room>

/// (U) friends_events <login> <password> <eventname>
/// lista znajomych uczestniczących w danym wydarzeniu
///  <login> <eventname> <friendlogin> 

/// (U) recommended_talks <login> <password> <start_timestamp> <end_timestamp> <limit>
/// zwraca referaty rozpoczynające się w podanym przedziale czasowym, które mogą zainteresować
/// danego uczestnika (zaproponuj parametr <score> obliczany na podstawie dostępnych danych
/// – ocen, obecności, znajomości itp.), wypisuje pierwsze <limit> referatów wg nalepszego <score>,
/// przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <speakerlogin> <start_timestamp> <title> <room> <score>

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
