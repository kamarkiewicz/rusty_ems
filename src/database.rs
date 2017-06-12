use errors::*;

use api::{Date, DateTime};
#[allow(unused_imports)]
use api::{AttendedTalk, UserPlan, DayPlan, BestTalk, MostPopularTalk, AbandonedTalk,
          RecentlyAddedTalk, RejectedTalk, Proposal, FriendsTalk, FriendsEvent, RecommendedTalk};

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

/// Sprawdza, czy połączona baza zawiera odpowiednią strukturę tabel.
/// Przeprowadza migrację jeśli jest taka potrzeba.
/// Pozyskuje zawartość skryptu tworzącego bazę w trakcie kompilacji.
pub fn setup_database(conn: &Connection) -> Result<()> {
    let up_sql = include_str!("../migrations/20170529191530_base/up.sql");
    let check_sql = include_str!("../migrations/20170529191530_base/check.sql");
    if conn.batch_execute(check_sql).is_err() {
        conn.batch_execute(up_sql)
            .expect("Unable to setup a database");
    }
    Ok(())
}

/// (*) organizer <secret> <newlogin> <newpassword>
/// tworzy uczestnika <newlogin> z uprawnieniami organizatora i hasłem <newpassword>,
/// argument <secret> musi być równy d8578edf8458ce06fbc5bb76a58c5ca4 // zwraca status OK/ERROR
pub fn create_organizer_account(conn: &Connection,
                                newlogin: String,
                                newpassword: String)
                                -> Result<()> {

    let query = r#"
        INSERT INTO persons (login, password, is_organizer)
        VALUES ($1, $2, TRUE)"#;
    conn.execute(query, &[&newlogin, &newpassword])
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

    authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)?;

    // insert new event
    let query = r#"
        INSERT INTO events (eventname, start_timestamp, end_timestamp)
        VALUES ($1, $2, $3)"#;
    conn.execute(query, &[&eventname, &start_timestamp, &end_timestamp])
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

    authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)?;

    // insert new person
    let query = r#"
        INSERT INTO persons (login, password, is_organizer)
        VALUES ($1, $2, FALSE)"#;
    conn.execute(query, &[&newlogin, &newpassword])
        .chain_err(|| "Unable to insert User person")?;

    Ok(())
}

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "talk_status")]
pub enum TalkStatus {
    Proposed,
    Accepted,
    Rejected,
}

/// (*O) talk <login> <password>
///     <speakerlogin> <talk> <title> <start_timestamp> <room> <initial_evaluation> <eventname>
/// rejestracja referatu/zatwierdzenie referatu spontanicznego,
/// <talk> jest unikalnym identyfikatorem referatu,
/// <initial_evaluation> jest oceną organizatora w skali 0-10 – jest to ocena traktowana
///     tak samo jak ocena uczestnika obecnego na referacie,
/// <eventname> jest nazwą wydarzenia, którego częścią jest dany referat - może być pustym
///     napisem, co oznacza, że referat nie jest przydzielony do jakiegokolwiek wydarzenia
///
/// Referat, który jest przypisywany do wydarzenia, musi zaczynać się w czasie trwania wydarzenia.
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

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)?;
    let speaker_id = authorize_person_as(conn, &speakerlogin, None, PersonType::Whatever)?;
    let event_id: Option<i32> = if !eventname.is_empty() {
        let query = r#"
            SELECT id FROM events
            WHERE eventname = $1
              AND start_timestamp <= $2
              AND end_timestamp >= $2
            LIMIT 1"#;
        conn.query(query, &[&eventname, &start_timestamp])
            .chain_err(|| "Unable to load event")?
            .iter()
            .map(|row| row.get("id"))
            .next()
            .ok_or_else(|| format!("event with eventname=`{}` not found", eventname))?
    } else {
        None
    };

    // upsert a talk
    let query = r#"
        INSERT INTO talks (talk, speaker_id, status, title, start_timestamp, room, event_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (talk) DO UPDATE SET
          speaker_id = EXCLUDED.speaker_id,
          status = EXCLUDED.status,
          title = EXCLUDED.title,
          start_timestamp = EXCLUDED.start_timestamp,
          room = EXCLUDED.room,
          event_id = EXCLUDED.event_id,
          modified_at = now()
        RETURNING id"#;
    let talk_id: i32 = conn.query(query,
                                  &[&talk,
                                    &speaker_id,
                                    &TalkStatus::Accepted,
                                    &title,
                                    &start_timestamp,
                                    &room,
                                    &event_id])
        .chain_err(|| "Unable to upsert a talk")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| format!("talk with talk=`{}` not found", talk))?;

    // initial evaluation
    let query = r#"
        INSERT INTO person_rated_talk (person_id, talk_id, rating)
        VALUES ($1, $2, $3)"#;
    conn.execute(query, &[&person_id, &talk_id, &initial_evaluation])
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

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    let query = r#"
        SELECT id FROM events
        WHERE eventname = $1
        LIMIT 1"#;
    let event_id: i32 =
        conn.query(query, &[&eventname])
            .chain_err(|| "Unable to load event")?
            .iter()
            .map(|row| row.get("id"))
            .next()
            .ok_or_else(|| format!("event with eventname=`{}` not found", eventname))?;

    let query = r#"
        INSERT INTO person_registered_for_event (person_id, event_id)
        VALUES ($1, $2)"#;
    conn.execute(query, &[&person_id, &event_id])
        .chain_err(|| "Person can't be registered for event")?;

    Ok(())
}

/// (*U) attendance <login> <password> <talk>
/// odnotowanie faktycznej obecności uczestnika <login> na referacie <talk>
///
/// Uczestnik mógł być tylko na zatwierdzonym referacie.
pub fn attendance(conn: &Connection, login: String, password: String, talk: String) -> Result<()> {

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    let query = r#"
        SELECT id FROM talks
        WHERE talk = $1
          AND status = $2
        LIMIT 1"#;
    let talk_id: i32 = conn.query(query, &[&talk, &TalkStatus::Accepted])
        .chain_err(|| "Unable to load the talk")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| format!("talk with talk=`{}` not found", talk))?;

    let query = r#"
        INSERT INTO person_attended_for_talk (person_id, talk_id)
        VALUES ($1, $2)"#;
    conn.execute(query, &[&person_id, &talk_id])
        .chain_err(|| "Unable to mark attendance of User for the talk")?;

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

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    let query = r#"
        SELECT id FROM talks
        WHERE talk = $1
          AND status = $2
        LIMIT 1"#;
    let talk_id: i32 = conn.query(query, &[&talk, &TalkStatus::Accepted])
        .chain_err(|| "Unable to load the talk")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| "talk doesn't exist or is not accepted")?;

    let query = r#"
        INSERT INTO person_rated_talk (person_id, talk_id, rating)
        VALUES ($1, $2, $3)"#;
    conn.execute(query, &[&person_id, &talk_id, &rating])
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

    authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)?;

    // update a proposal
    let query = r#"
        UPDATE talks
        SET status = $1
        WHERE talk = $2
          AND status = $3"#;
    let updates = conn.execute(query,
                               &[&TalkStatus::Rejected, &talk, &TalkStatus::Proposed])
        .chain_err(|| "Unable to reject a proposal")?;
    if updates != 1 {
        bail!("There was no proposal to reject")
    }

    Ok(())
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

    let speaker_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    // insert a new proposal
    let query = r#"
        INSERT INTO talks (speaker_id, talk, status, title, start_timestamp)
        VALUES ($1, $2, $3, $4, $5)"#;
    conn.execute(query,
                 &[&speaker_id,
                   &talk,
                   &TalkStatus::Proposed,
                   &title,
                   &start_timestamp])
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

    let person1_id = authorize_person_as(conn, &login1, Some(&password), PersonType::User)?;
    let person2_id = authorize_person_as(conn, &login2, None, PersonType::User)?;

    let query = r#"
        INSERT INTO person_knows_person (person1_id, person2_id)
        VALUES ($1, $2)"#;
    conn.execute(query, &[&person1_id, &person2_id])
        .chain_err(|| "These users are already friends?")?;

    Ok(())
}

/// (*N) user_plan <login> <limit>
/// zwraca plan najbliższych referatów z wydarzeń, na które dany uczestnik jest zapisany
/// (wg rejestracji na wydarzenia) posortowany wg czasu rozpoczęcia,
/// wypisuje pierwsze <limit> referatów, przy czym 0 oznacza, że należy wypisać wszystkie
/// Atrybuty zwracanych krotek:
///   <login> <talk> <start_timestamp> <title> <room>
pub fn user_plan(conn: &Connection, login: String, limit: u32) -> Result<Vec<UserPlan>> {

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };

    let query = format!(r#"
        WITH cte(person_id, speakerlogin, talk, start_timestamp, title, room) AS (
          SELECT person_id, login, talk, start_timestamp, title, room
          FROM person_registered_for_event prfe
            JOIN talks USING(event_id)
            JOIN persons speakers ON speaker_id=speakers.id
          WHERE status = $1
            AND start_timestamp >= now()
        )
        SELECT speakerlogin, talk, start_timestamp, title, room
        FROM cte
          JOIN persons ON cte.person_id = persons.id
        WHERE persons.login = $2
        ORDER BY start_timestamp
        {}"#,
                        limit);
    let plans: Vec<_> = conn.query(&query[..], &[&TalkStatus::Accepted, &login])
        .chain_err(|| "Unable to load person's plan")?
        .iter()
        .map(|row| {
            UserPlan {
                login: row.get("speakerlogin"),
                talk: row.get("talk"),
                start_timestamp: row.get("start_timestamp"),
                title: row.get("title"),
                room: row.get("room"),
            }
        })
        .collect();

    Ok(plans)
}

/// (*N) day_plan <timestamp>
/// zwraca listę wszystkich referatów zaplanowanych na dany dzień posortowaną rosnąco wg sal,
///     w drugiej kolejności wg czasu rozpoczęcia
///  <talk> <start_timestamp> <title> <room>
pub fn day_plan(conn: &Connection, date: Date) -> Result<Vec<DayPlan>> {

    let query = r#"
        SELECT talk, start_timestamp, title, room
        FROM talks
        WHERE status = $1
          AND start_timestamp::date = $2
        ORDER BY room, start_timestamp"#;
    let plans: Vec<_> = conn.query(&query[..], &[&TalkStatus::Accepted, &date])
        .chain_err(|| "Unable to load day plan")?
        .iter()
        .map(|row| {
                 DayPlan {
                     talk: row.get("talk"),
                     start_timestamp: row.get("start_timestamp"),
                     title: row.get("title"),
                     room: row.get("room"),
                 }
             })
        .collect();

    Ok(plans)
}

/// (*N) best_talks <start_timestamp> <end_timestamp> <limit> <all>
/// zwraca referaty rozpoczynające się w  danym przedziale czasowym posortowane malejąco
/// wg średniej oceny uczestników, przy czym jeśli <all> jest równe 1 należy wziąć
/// pod uwagę wszystkie oceny, w przeciwnym przypadku tylko oceny uczestników, którzy
/// byli na referacie obecni, wypisuje pierwsze <limit> referatów, przy czym 0 oznacza,
/// że należy wypisać wszystkie
///  <talk> <start_timestamp> <title> <room>
pub fn best_talks(conn: &Connection,
                  start_timestamp: DateTime,
                  end_timestamp: DateTime,
                  limit: u32,
                  all: bool)
                  -> Result<Vec<BestTalk>> {

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };
    let all = if all {
        ""
    } else {
        r#"WHERE is_organizer = TRUE
             OR (person_id, talk_id) IN (SELECT * FROM person_attended_for_talk)"#
    };

    let query = format!(r#"
        WITH cte(talk_id, average_rate) AS (
          SELECT talk_id, avg(rating)
          FROM person_rated_talk
            JOIN persons ON persons.id = person_id
            JOIN talks ON talks.id = talk_id
            JOIN events ON events.id = event_id
          {}
          GROUP BY talk_id
        )
        SELECT talk, start_timestamp, title, room
        FROM talks
          JOIN cte ON cte.talk_id = talks.id
        WHERE status = $1
          AND start_timestamp >= $2
          AND start_timestamp <= $3
        ORDER BY average_rate DESC
        {}"#,
                        all,
                        limit);
    let talks: Vec<_> = conn.query(&query[..],
                                   &[&TalkStatus::Accepted, &start_timestamp, &end_timestamp])
        .chain_err(|| "Unable to load best talks")?
        .iter()
        .map(|row| {
                 BestTalk {
                     talk: row.get("talk"),
                     start_timestamp: row.get("start_timestamp"),
                     title: row.get("title"),
                     room: row.get("room"),
                 }
             })
        .collect();

    Ok(talks)
}

/// (*N) most_popular_talks <start_timestamp> <end_timestamp> <limit>
/// zwraca referaty rozpoczynające się w podanym przedziału czasowego posortowane malejąco
/// wg obecności, wypisuje pierwsze <limit> referatów, przy czym 0 oznacza,
/// że należy wypisać wszystkie
///  <talk> <start_timestamp> <title> <room>
pub fn most_popular_talks(conn: &Connection,
                          start_timestamp: DateTime,
                          end_timestamp: DateTime,
                          limit: u32)
                          -> Result<Vec<MostPopularTalk>> {

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };
    let query = format!(r#"
        WITH cte(talk_id, arrivals) AS (
          SELECT talk_id, COUNT(person_id)
          FROM person_attended_for_talk
          GROUP BY talk_id
        )
        SELECT talk, start_timestamp, title, room
        FROM talks
          JOIN cte ON cte.talk_id = talks.id
        WHERE status = $1
          AND start_timestamp >= $2
          AND start_timestamp <= $3
        ORDER BY arrivals DESC
        {}"#,
                        limit);
    let talks: Vec<_> = conn.query(&query[..],
                                   &[&TalkStatus::Accepted, &start_timestamp, &end_timestamp])
        .chain_err(|| "Unable to load most popular talks")?
        .iter()
        .map(|row| {
                 MostPopularTalk {
                     talk: row.get("talk"),
                     start_timestamp: row.get("start_timestamp"),
                     title: row.get("title"),
                     room: row.get("room"),
                 }
             })
        .collect();

    Ok(talks)
}

/// (*U) attended_talks <login> <password>
/// zwraca dla danego uczestnika referaty, na których był obecny
///  <talk> <start_timestamp> <title> <room>
pub fn attended_talks(conn: &Connection,
                      login: String,
                      password: String)
                      -> Result<Vec<AttendedTalk>> {

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    let query = r#"
        SELECT talk, start_timestamp, title, room
        FROM person_attended_for_talk paft
          JOIN talks ON paft.talk_id = talks.id
        WHERE paft.person_id = $1"#;
    let talks: Vec<_> = conn.query(query, &[&person_id])
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

    Ok(talks)
}

/// (*O) abandoned_talks <login> <password> <limit>
/// zwraca listę referatów posortowaną malejąco wg liczby uczestników
/// <number> zarejestrowanych na wydarzenie obejmujące referat,
/// którzy nie byli na tym referacie obecni, wypisuje pierwsze <limit> referatów,
/// przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <start_timestamp> <title> <room> <number>
pub fn abandoned_talks(conn: &Connection,
                       login: String,
                       password: String,
                       limit: u32)
                       -> Result<Vec<AbandonedTalk>> {

    authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)?;

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };

    let query = format!(r#"
        WITH person_registered_for_talk(person_id, talk_id) AS (
          SELECT person_id, talks.id
          FROM person_registered_for_event
            JOIN talks USING(event_id)
        ),
        cte(talk_id, absent) AS (
          SELECT talk_id, count(person_id)
          FROM person_registered_for_talk
          WHERE (person_id, talk_id) NOT IN (SELECT * FROM person_attended_for_talk)
          GROUP BY talk_id
        )
        SELECT talk, start_timestamp, title, room, absent
        FROM talks
          JOIN cte ON cte.talk_id = talks.id
        ORDER BY absent DESC
        {}"#,
                        limit);
    let talks: Vec<_> = conn.query(&query[..], &[])
        .chain_err(|| "Unable to load abandoned talks")?
        .iter()
        .map(|row| {
            AbandonedTalk {
                talk: row.get("talk"),
                start_timestamp: row.get("start_timestamp"),
                title: row.get("title"),
                room: row.get("room"),
                number: row.get("absent"),
            }
        })
        .collect();

    Ok(talks)
}

/// (N) recently_added_talks <limit>
/// zwraca listę ostatnio zarejestrowanych referatów, wypisuje ostatnie <limit> referatów
/// wg daty zarejestrowania, przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <speakerlogin> <start_timestamp> <title> <room>
pub fn recently_added_talks(conn: &Connection, limit: u32) -> Result<Vec<RecentlyAddedTalk>> {

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };

    let query = format!(r#"
        SELECT talk, login AS speakerlogin, start_timestamp, title, room
        FROM talks
          JOIN persons ON persons.id = talks.speaker_id
        WHERE status = $1
        ORDER BY modified_at DESC
        {}"#,
                        limit);
    let talks: Vec<_> = conn.query(&query[..], &[&TalkStatus::Accepted])
        .chain_err(|| "Unable to load recently added talks")?
        .iter()
        .map(|row| {
            RecentlyAddedTalk {
                talk: row.get("talk"),
                speakerlogin: row.get("speakerlogin"),
                start_timestamp: row.get("start_timestamp"),
                title: row.get("title"),
                room: row.get("room"),
            }
        })
        .collect();

    Ok(talks)
}

/// (U/O) rejected_talks <login> <password>
/// jeśli wywołujący ma uprawnienia organizatora zwraca listę wszystkich odrzuconych referatów
/// spontanicznych, w przeciwnym przypadku listę odrzuconych referatów wywołującego ją uczestnika
///  <talk> <speakerlogin> <start_timestamp> <title>
pub fn rejected_talks(conn: &Connection,
                      login: String,
                      password: String)
                      -> Result<Vec<RejectedTalk>> {

    let (typ, person_id) =
        authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)
            .map(|id| (PersonType::Organizer, id))
            .or(authorize_person_as(conn, &login, Some(&password), PersonType::User)
                    .map(|id| (PersonType::User, id)))?;

    let talks: Vec<_> = match typ {
            PersonType::Organizer => {
                let query = r#"
                SELECT talk, login AS speakerlogin, start_timestamp, title
                FROM talks
                  JOIN persons ON persons.id = talks.speaker_id
                WHERE status = $1"#;
                conn.query(&query[..], &[&TalkStatus::Rejected])
            }
            PersonType::User => {
                let query = r#"
                SELECT talk, login AS speakerlogin, start_timestamp, title
                FROM talks
                  JOIN persons ON persons.id = talks.speaker_id
                WHERE status = $1
                  AND talks.speaker_id = $2"#;
                conn.query(&query[..], &[&TalkStatus::Rejected, &person_id])
            }
            PersonType::Whatever => unreachable!(),
        }
        .chain_err(|| "Unable to load rejected talks")?
        .iter()
        .map(|row| {
                 RejectedTalk {
                     talk: row.get("talk"),
                     speakerlogin: row.get("speakerlogin"),
                     start_timestamp: row.get("start_timestamp"),
                     title: row.get("title"),
                 }
             })
        .collect();

    Ok(talks)
}

/// (O) proposals <login> <password>
/// zwraca listę propozycji referatów spontanicznych do zatwierdzenia lub odrzucenia,
/// zatwierdzenie lub odrzucenie referatu polega na wywołaniu przez organizatora
/// funkcji talk lub reject z odpowiednimi parametrami
///  <talk> <speakerlogin> <start_timestamp> <title>
pub fn proposals(conn: &Connection, login: String, password: String) -> Result<Vec<Proposal>> {

    authorize_person_as(conn, &login, Some(&password), PersonType::Organizer)?;

    let query = r#"
        SELECT talk, login AS speakerlogin, start_timestamp, title
        FROM talks
          JOIN persons ON persons.id = talks.speaker_id
        WHERE status = $1"#;
    let talks: Vec<_> = conn.query(query, &[&TalkStatus::Proposed])
        .chain_err(|| "Unable to load proposals")?
        .iter()
        .map(|row| {
                 Proposal {
                     talk: row.get("talk"),
                     speakerlogin: row.get("speakerlogin"),
                     start_timestamp: row.get("start_timestamp"),
                     title: row.get("title"),
                 }
             })
        .collect();

    Ok(talks)
}

/// (U) friends_talks <login> <password> <start_timestamp> <end_timestamp> <limit>
/// lista referatów  rozpoczynających się w podanym przedziale czasowym wygłaszanych
/// przez znajomych danego uczestnika posortowana wg czasu rozpoczęcia,
/// wypisuje pierwsze <limit> referatów, przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <speakerlogin> <start_timestamp> <title> <room>
pub fn friends_talks(conn: &Connection,
                     login: String,
                     password: String,
                     start_timestamp: DateTime,
                     end_timestamp: DateTime,
                     limit: u32)
                     -> Result<Vec<FriendsTalk>> {

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    let limit = if limit == 0 {
        "".to_owned()
    } else {
        format!("LIMIT {}", limit)
    };

    let query = format!(r#"
        WITH friends(id) AS (
            SELECT person2_id
            FROM person_knows_person
            JOIN (
                SELECT person2_id AS person1_id, person1_id AS person2_id
                FROM person_knows_person
            ) AS reversed USING(person1_id, person2_id)
            WHERE person1_id = $1
        )
        SELECT talk, login AS speakerlogin, start_timestamp, title, room
        FROM talks
          JOIN persons ON persons.id = talks.speaker_id
          JOIN friends ON friends.id = talks.speaker_id
        WHERE status = $2
          AND start_timestamp >= $3
          AND start_timestamp <= $4
        ORDER BY start_timestamp
        {}"#,
                        limit);
    let talks: Vec<_> = conn.query(&query[..],
                                   &[&person_id,
                                     &TalkStatus::Accepted,
                                     &start_timestamp,
                                     &end_timestamp])
        .chain_err(|| "Unable to load friends talks")?
        .iter()
        .map(|row| {
            FriendsTalk {
                talk: row.get("talk"),
                speakerlogin: row.get("speakerlogin"),
                start_timestamp: row.get("start_timestamp"),
                title: row.get("title"),
                room: row.get("room"),
            }
        })
        .collect();

    Ok(talks)
}

/// (U) friends_events <login> <password> <eventname>
/// lista znajomych uczestniczących w danym wydarzeniu
///  <login> <eventname> <friendlogin>
pub fn friends_events(conn: &Connection,
                      login: String,
                      password: String,
                      eventname: String)
                      -> Result<Vec<FriendsEvent>> {

    let person_id = authorize_person_as(conn, &login, Some(&password), PersonType::User)?;

    let query = r#"
        WITH friends(id) AS (
            SELECT person2_id
            FROM person_knows_person
            JOIN (
                SELECT person2_id AS person1_id, person1_id AS person2_id
                FROM person_knows_person
            ) AS reversed USING(person1_id, person2_id)
            WHERE person1_id = $1
        )
        SELECT login AS friendlogin
        FROM person_registered_for_event
          JOIN friends ON friends.id = person_id
          JOIN events ON events.id = event_id
          JOIN persons ON persons.id = person_id"#;
    let talks: Vec<_> = conn.query(&query[..], &[&person_id])
        .chain_err(|| "Unable to load friends events")?
        .iter()
        .map(|row| {
                 FriendsEvent {
                     login: login.clone(),
                     eventname: eventname.clone(),
                     friendlogin: row.get("friendlogin"),
                 }
             })
        .collect();

    Ok(talks)
}

/// (U) recommended_talks <login> <password> <start_timestamp> <end_timestamp> <limit>
/// zwraca referaty rozpoczynające się w podanym przedziale czasowym, które mogą zainteresować
/// danego uczestnika (zaproponuj parametr <score> obliczany na podstawie dostępnych danych
/// – ocen, obecności, znajomości itp.), wypisuje pierwsze <limit> referatów wg nalepszego <score>,
/// przy czym 0 oznacza, że należy wypisać wszystkie
///  <talk> <speakerlogin> <start_timestamp> <title> <room> <score>

enum PersonType {
    Whatever,
    User,
    Organizer,
}

fn authorize_person_as(conn: &Connection,
                       login: &str,
                       password: Option<&str>,
                       person_type: PersonType)
                       -> Result<i32> {
    use self::PersonType::*;
    let organizer_part = match person_type {
        Whatever => "",
        User => "AND is_organizer=FALSE",
        Organizer => "AND is_organizer=TRUE",
    };

    match password {
            Some(ref password) => {
                let query = format!(r#"
                    SELECT id FROM persons
                    WHERE login = $1 AND password = $2
                      {}
                    LIMIT 1"#,
                                    organizer_part);
                conn.query(&query[..], &[&login, &password])
            }
            None => {
                let query = format!(r#"
                    SELECT id FROM persons
                    WHERE login = $1
                      {}
                    LIMIT 1"#,
                                    organizer_part);
                conn.query(&query[..], &[&login])
            }
        }
        .chain_err(|| "Unable to authorize person")?
        .iter()
        .map(|row| row.get("id"))
        .next()
        .ok_or_else(|| "Requested person not found".into())
}
