use errors::*;
use api::*;
use database::*;

pub struct Context {
    conn: Option<PgConnection>,
}

#[allow(unused_variables)]
#[allow(unused_imports)]
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
                   create_event(&conn, login, password,
                        eventname, start_timestamp, end_timestamp)?;
                   Response::Ok(None)
               },

               Request::User {
                   login,
                   password,
                   newlogin,
                   newpassword
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   create_user(&conn, login, password,
                        newlogin, newpassword)?;
                   Response::Ok(None)
               },

               Request::Talk {
                   login,
                   password,
                   speakerlogin,
                   talk,
                   title,
                   start_timestamp,
                   room,
                   initial_evaluation,
                   eventname
               } => { Response::NotImplemented },

               Request::RegisterUserForEvent {
                   login,
                   password,
                   eventname
               } => { Response::NotImplemented },

               Request::Attendance {
                   login,
                   password,
                   talk
               } => { Response::NotImplemented },

               Request::Evaluation {
                   login,
                   password,
                   talk,
                   rating
               } => { Response::NotImplemented },

               Request::Reject {
                   login,
                   password,
                   talk
               } => { Response::NotImplemented },

               Request::Proposal {
                   login,
                   password,
                   talk,
                   title,
                   start_timestamp
               } => { Response::NotImplemented },

               Request::Friends {
                   login1,
                   password,
                   login2
               } => { Response::NotImplemented },

               Request::UserPlan { login, limit } => { Response::NotImplemented },

               Request::DayPlan { timestamp } => { Response::NotImplemented },

               Request::BestTalks {
                   start_timestamp,
                   end_timestamp,
                   limit,
                   all
               } => { Response::NotImplemented },

               Request::MostPopularTalks {
                   start_timestamp,
                   end_timestamp,
                   limit
               } => { Response::NotImplemented },

               Request::AttendedTalks { login, password } => { Response::NotImplemented },

               Request::AbandonedTalks {
                   login,
                   password,
                   limit
               } => { Response::NotImplemented },

               Request::RecentlyAddedTalks { limit } => { Response::NotImplemented },

               Request::RejectedTalks { login, password } => { Response::NotImplemented },

               Request::Proposals { login, password } => { Response::NotImplemented },

               Request::FriendsTalks {
                   login,
                   password,
                   start_timestamp,
                   end_timestamp,
                   limit
               } => { Response::NotImplemented },

               Request::FriendsEvents {
                   login,
                   password,
                   event
               } => { Response::NotImplemented },

               Request::RecommendedTalks {
                   login,
                   password,
                   start_timestamp,
                   end_timestamp,
                   limit
               } => { Response::NotImplemented },

           })
    }
}
