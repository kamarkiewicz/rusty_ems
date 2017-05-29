use errors::*;

use diesel;
use diesel::prelude::*;
pub use diesel::pg::PgConnection;

pub fn establish_connection(login: String, password: String, baza: String) -> Result<PgConnection> {
    let database_url = format!("postgres://{}:{}@localhost/{}", login, password, baza);
    PgConnection::establish(&database_url)
        .chain_err(|| format!("Error connecting to {}", database_url))
}

pub fn create_organizer_account(conn: &PgConnection,
                                newlogin: String,
                                newpassword: String)
                                -> Result<()> {
    use schema::users;
    use models::{User, NewUser};

    let organizer = NewUser {
        login: newlogin.as_ref(),
        password: newpassword.as_ref(),
        is_organizer: true,
    };

    diesel::insert(&organizer)
        .into(users::table)
        .get_result::<User>(conn)
        .chain_err(|| "unable to add organizer user to database")?;

    Ok(())
}
