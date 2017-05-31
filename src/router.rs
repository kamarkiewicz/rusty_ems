use errors::*;
use api::*;
use database::*;

pub struct Context {
    conn: Option<PgConnection>,
}

impl Context {
    pub fn new() -> Context {
        Context { conn: None }
    }

    pub fn resolve(&mut self, req: Request) -> Result<Response> {
        Ok(match req {
               Request::Open {
                   login,
                   password,
                   baza,
               } => {
                   self.conn = Some(establish_connection(login, password, baza)?);
                   Response::Ok(None)
               },
               Request::Organizer {
                   newlogin,
                   newpassword,
                   .. // secret has been validated on Request creation
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                    create_organizer_account(&conn, newlogin, newpassword)?;
                    Response::Ok(None)
               },
               Request::Event {
                   login,
                   password,
                   eventname,
                   start_timestamp,
                   end_timestamp
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   //let person = authorize_person(&conn, login, password, PersonAs::Organizer)?;
                   create_event(&conn, login, password,
                        eventname, start_timestamp, end_timestamp)?;
                   Response::Ok(None)
               }
               _ => Response::NotImplemented,
           })
    }
}
