use errors::*;

use api::{DateTime, AttendedTalk, UserPlan};

pub use postgres::{Connection, TlsMode};

/// (*) open <baza> <login> <password>
/// przekazuje dane umożliwiające podłączenie Twojego programu do bazy - nazwę bazy,
/// login oraz hasło, wywoływane dokładnie jeden raz, w pierwszej linii wejścia
/// zwraca status OK/ERROR w zależności od tego czy udało się nawiązać połączenie z bazą
pub fn establish_connection(login: String, password: String, baza: String) -> Result<Connection> {
    let database_url = format!("postgres://{}:{}@localhost/{}", login, password, baza);
    Connection::connect(database_url.as_ref(), TlsMode::None)
        .chain_err(|| format!("Error connecting to {}", database_url))
}

/// (*) organizer <secret> <newlogin> <newpassword>
/// tworzy uczestnika <newlogin> z uprawnieniami organizatora i hasłem <newpassword>,
/// argument <secret> musi być równy d8578edf8458ce06fbc5bb76a58c5ca4 // zwraca status OK/ERROR
pub fn create_organizer_account(conn: &Connection,
                                newlogin: String,
                                newpassword: String)
                                -> Result<()> {
    conn.execute(r#"
            INSERT INTO persons (login, password, is_organizer)
            VALUES ($1, $2, TRUE)"#,
                 &[&newlogin, &newpassword])
        .chain_err(|| "Unable to insert organizer person")?;
    Ok(())
}

/// (*O) event <login> <password> <eventname> <start_timestamp> <end_timestamp>
/// rejestracja wydarzenia, napis <eventname> jest unikalny
pub fn create_event(conn: &Connection,
                    login: String,
                    password: String,
                    eventname: String,
                    start_timestamp: DateTime,
                    end_timestamp: DateTime)
                    -> Result<()> {
    authorize_person_as(conn, login, Some(password), PersonType::Organizer)?;

    // insert new event
    conn.execute(r#"
            INSERT INTO events (eventname, start_timestamp, end_timestamp)
            VALUES ($1, $2, $3)"#,
                 &[&eventname, &start_timestamp, &end_timestamp])
        .chain_err(|| "Unable to insert event")?;

    Ok(())
}

/// (*O) user <login> <password> <newlogin> <newpassword>
/// rejestracja nowego uczestnika
/// <login> i <password> służą do autoryzacji wywołującego funkcję,
/// który musi posiadać uprawnienia organizatora,
/// <newlogin> <newpassword> są danymi nowego uczestnika,
/// <newlogin> jest unikalny
pub fn create_user(conn: &Connection,
                   login: String,
                   password: String,
                   newlogin: String,
                   newpassword: String)
                   -> Result<()> {
    authorize_person_as(conn, login, Some(password), PersonType::Organizer)?;

    // insert new person
    conn.execute(r#"
            INSERT INTO persons (login, password, is_organizer)
            VALUES ($1, $2, FALSE)"#,
                 &[&newlogin, &newpassword])
        .chain_err(|| "Unable to insert participant person")?;

    Ok(())
}

pub enum TalkStatus {
    Proposed,
    Accepted,
    Rejected,
}

impl From<TalkStatus> for i16 {
    fn from(status: TalkStatus) -> Self {
        use self::TalkStatus::*;
        match status {
            Proposed => 0,
            Accepted => 1,
            Rejected => 2,
        }
    }
}

/// (*O) talk <login> <password>
///     <speakerlogin> <talk> <title> <start_timestamp> <room> <initial_evaluation> <eventname>
/// rejestracja referatu/zatwierdzenie referatu spontanicznego,
/// <talk> jest unikalnym identyfikatorem referatu,
/// <initial_evaluation> jest oceną organizatora w skali 0-10 – jest to ocena traktowana
///     tak samo jak ocena uczestnika obecnego na referacie,
/// <eventname> jest nazwą wydarzenia, którego częścią jest dany referat - może być pustym
///     napisem, co oznacza, że referat nie jest przydzielony do jakiegokolwiek wydarzenia
pub fn register_or_accept_talk(conn: &Connection,
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
    let person_id = authorize_person_as(conn, login, Some(password), PersonType::Organizer)?;

    let speaker_id = authorize_person_as(conn, speakerlogin, None, PersonType::Whatever)?;

    let event_id: Option<i32> = if eventname.is_empty() {
        None
    } else {
        conn.query(r#"
            SELECT id FROM events WHERE eventname=$1 LIMIT 1"#,
                   &[&eventname])
            .chain_err(|| "Unable to load event")?
            .iter()
            .map(|row| row.get("id"))
            .next()
            .ok_or_else(|| format!("event with eventname=`{}` not found", eventname))?
    };

    // TODO: handle accepting the proposal
    // insert a new talk
    let status: i16 = TalkStatus::Accepted.into();
    let talk_id: i32 = conn.query(r#"
            INSERT INTO talks (speaker_id, talk, status, title, start_timestamp, room, event_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id"#,
                                  &[&speaker_id,
                                    &talk,
                                    &status,
                                    &title,
                                    &start_timestamp,
                                    &room,
                                    &event_id])
        .chain_err(|| "Unable to insert a talk")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| format!("talk with talk=`{}` not found", talk))?;

    // initial evaluation
    conn.execute(r#"
            INSERT INTO person_rated_talk (person_id, talk_id, rating)
            VALUES ($1, $2, $3)"#,
                 &[&person_id, &talk_id, &initial_evaluation])
        .chain_err(|| "Unable to evaluate the talk")?;


    Ok(())
}

/// (*U) register_user_for_event <login> <password> <eventname>
/// rejestracja uczestnika <login> na wydarzenie <eventname>
pub fn register_user_for_event(conn: &Connection,
                               login: String,
                               password: String,
                               eventname: String)
                               -> Result<()> {
    let person_id = authorize_person_as(conn, login, Some(password), PersonType::Participant)?;

    let event_id: i32 =
        conn.query(r#"
            SELECT id FROM events WHERE eventname=$1 LIMIT 1"#,
                   &[&eventname])
            .chain_err(|| "Unable to load event")?
            .iter()
            .map(|row| row.get("id"))
            .next()
            .ok_or_else(|| format!("event with eventname=`{}` not found", eventname))?;

    conn.execute(r#"
            INSERT INTO person_registered_for_event (person_id, event_id)
            VALUES ($1, $2)"#,
                 &[&person_id, &event_id])
        .chain_err(|| "Person can't be registered for event")?;

    Ok(())
}

/// (*U) attendance <login> <password> <talk>
/// odnotowanie faktycznej obecności uczestnika <login> na referacie <talk>
pub fn attendance(conn: &Connection, login: String, password: String, talk: String) -> Result<()> {
    let person_id = authorize_person_as(conn, login, Some(password), PersonType::Participant)?;

    let talk_id: i32 = conn.query(r#"
            SELECT id FROM talks WHERE talk=$1 LIMIT 1"#,
                                  &[&talk])
        .chain_err(|| "Unable to load the talk")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| format!("talk with talk=`{}` not found", talk))?;

    conn.execute(r#"
            INSERT INTO person_attended_for_talk (person_id, talk_id)
            VALUES ($1, $2)"#,
                 &[&person_id, &talk_id])
        .chain_err(|| "Unable to mark attendance of participant for the talk")?;

    Ok(())
}

/// (*U) evaluation <login> <password> <talk> <rating>
/// ocena referatu <talk> w skali 0-10 przez uczestnika <login>
pub fn evaluation(conn: &Connection,
                  login: String,
                  password: String,
                  talk: String,
                  rating: i16)
                  -> Result<()> {
    let person_id = authorize_person_as(conn, login, Some(password), PersonType::Participant)?;

    let talk_id: i32 = conn.query(r#"
            SELECT id FROM talks WHERE talk=$1 LIMIT 1"#,
                                  &[&talk])
        .chain_err(|| "Unable to load the talk")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| format!("talk with talk=`{}` not found", talk))?;

    conn.execute(r#"
            INSERT INTO person_rated_talk (person_id, talk_id, rating)
            VALUES ($1, $2, $3)"#,
                 &[&person_id, &talk_id, &rating])
        .chain_err(|| "Unable to evaluate the talk")?;

    Ok(())
}

/// (O) reject <login> <password> <talk>
/// usuwa referat spontaniczny <talk> z listy zaproponowanych,
pub fn reject_spontaneous_talk(conn: &Connection,
                               login: String,
                               password: String,
                               talk: String)
                               -> Result<()> {
    authorize_person_as(conn, login, Some(password), PersonType::Organizer)?;

    let rejected: i16 = TalkStatus::Rejected.into();
    let proposed: i16 = TalkStatus::Proposed.into();

    // update a proposal
    let updates = conn.execute(r#"
            UPDATE talks SET status = $1 WHERE talk = $2 AND status = $3"#,
                               &[&rejected, &talk, &proposed])
        .chain_err(|| "Unable to reject a proposal")?;

    if updates == 1 {
        Ok(())
    } else {
        Err("There was no proposal to reject".into())
    }
}

/// (U) proposal <login> <password> <talk> <title> <start_timestamp>
/// propozycja referatu spontanicznego, <talk> - unikalny identyfikator referatu
pub fn propose_spontaneous_talk(conn: &Connection,
                                login: String,
                                password: String,
                                talk: String,
                                title: String,
                                start_timestamp: DateTime)
                                -> Result<()> {
    let speaker_id = authorize_person_as(conn, login, Some(password), PersonType::Participant)?;
    let status: i16 = TalkStatus::Proposed.into();

    // insert a new proposal
    conn.execute(r#"
            INSERT INTO talks (speaker_id, talk, status, title, start_timestamp)
            VALUES ($1, $2, $3, $4, $5)"#,
                 &[&speaker_id, &talk, &status, &title, &start_timestamp])
        .chain_err(|| "Unable to insert a proposal")?;

    Ok(())
}

/// (U) friends <login1> <password> <login2>
/// uczestnik <login1> chce nawiązać znajomość z uczestnikiem <login2>, znajomość uznajemy
/// za nawiązaną jeśli obaj uczestnicy chcą ją nawiązać tj. po wywołaniach
/// friends <login1> <password1> <login2> i friends <login2> <password2> <login1>
pub fn make_friends(conn: &Connection,
                    login1: String,
                    password: String,
                    login2: String)
                    -> Result<()> {
    let person1_id = authorize_person_as(conn, login1, Some(password), PersonType::Participant)?;
    let person2_id = authorize_person_as(conn, login2, None, PersonType::Participant)?;

    conn.execute(r#"
            INSERT INTO person_knows_person (person1_id, person2_id)
            VALUES ($1, $2)"#,
                 &[&person1_id, &person2_id])
        .chain_err(|| "These participants cannot be friends")?;

    Ok(())
}

/// (*N) user_plan <login> <limit>
/// zwraca plan najbliższych referatów z wydarzeń, na które dany uczestnik jest zapisany
/// (wg rejestracji na wydarzenia) posortowany wg czasu rozpoczęcia,
/// wypisuje pierwsze <limit> referatów, przy czym 0 oznacza, że należy wypisać wszystkie
/// Atrybuty zwracanych krotek:
///   <login> <talk> <start_timestamp> <title> <room>
pub fn user_plan(conn: &Connection, login: String, limit: u32) -> Result<Vec<UserPlan>> {
    let person_id = authorize_person_as(conn, login.clone(), None, PersonType::Whatever)?;

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };

    let status: i16 = TalkStatus::Accepted.into();
    let query = format!(r#"
            SELECT persons.login as login, talk, talks.start_timestamp, title, room
            FROM person_registered_for_event prfe
                JOIN events ON prfe.event_id=events.id
                JOIN talks ON events.id=talks.event_id
                JOIN persons ON speaker_id=persons.id
            WHERE prfe.person_id = $1 AND talks.status = $2 {}"#,
                        limit);
    let user_plans: Vec<_> = conn.query(&query[..], &[&person_id, &status])
        .chain_err(|| "Unable to load person's plan")?
        .iter()
        .map(|row| {
            UserPlan {
                login: row.get("login"),
                talk: row.get("talk"),
                start_timestamp: row.get("start_timestamp"),
                title: row.get("title"),
                room: row.get("room"),
            }
        })
        .collect();

    Ok(user_plans)
}


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
pub fn attended_talks(conn: &Connection,
                      login: String,
                      password: String)
                      -> Result<Vec<AttendedTalk>> {
    let person_id = authorize_person_as(conn, login, Some(password), PersonType::Participant)?;

    let attended_talks: Vec<_> = conn.query(r#"
            SELECT talk, start_timestamp, title, room
            FROM person_attended_for_talk paft JOIN talks ON paft.talk_id=talks.id
            WHERE paft.person_id = $1"#,
                                            &[&person_id])
        .chain_err(|| "Unable to load person's talks")?
        .iter()
        .map(|row| {
                 AttendedTalk {
                     talk: row.get("talk"),
                     start_timestamp: row.get("start_timestamp"),
                     title: row.get("title"),
                     room: row.get("room"),
                 }
             })
        .collect();

    Ok(attended_talks)
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

enum PersonType {
    Whatever,
    Participant,
    Organizer,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn authorize_person_as(conn: &Connection,
                       login: String,
                       password: Option<String>,
                       person_type: PersonType)
                       -> Result<i32> {
    use self::PersonType::*;
    let organizer_part = match person_type {
        Whatever => "",
        Participant => "AND is_organizer=FALSE",
        Organizer => "AND is_organizer=TRUE",
    };

    match password {
        Some(password) => conn.query(&format!(
                r#"SELECT id FROM persons WHERE login=$1 AND password=$2 {} LIMIT 1"#,
                    organizer_part)[..],
                &[&login, &password]
        ),
        None => conn.query(&format!(
                r#"SELECT id FROM persons WHERE login=$1 {} LIMIT 1"#,
                    organizer_part)[..],
                &[&login]
        ),
    }
    .chain_err(|| "Unable to authorize person")?
    .iter()
    .map(|row| row.get("id"))
    .next()
    .ok_or_else(|| "Requested person not found".into())
}
