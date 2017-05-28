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
                   let conn = self.conn.as_ref().ok_or("")?;
                    create_organizer_account(&conn, newlogin, newpassword)?;
                    Response::Ok(None)
               }
               _ => Response::NotImplemented,
           })
    }
}
