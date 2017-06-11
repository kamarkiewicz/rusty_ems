use errors::*;
use api::*;
use database::*;

pub struct Context {
    conn: Option<Connection>,
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
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::Organizer {
                   newlogin,
                   newpassword,
                   .. // secret has been validated on Request creation
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                    create_organizer_account(&conn, newlogin, newpassword)
                        .chain_err(|| "during Request::Organizer")?;
                    Response::Ok(ResponseInfo::Empty)
               },

               Request::Event {
                   login,
                   password,
                   eventname,
                   start_timestamp,
                   end_timestamp
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let start_timestamp = match start_timestamp {
                       Timestamp::Date(d) => d.and_hms(0, 0, 0),
                       Timestamp::DateTime(dt) => dt
                   };
                   let end_timestamp = match end_timestamp {
                       Timestamp::Date(d) => d.and_hms(23, 59, 59),
                       Timestamp::DateTime(dt) => dt
                   };
                   create_event(&conn, login, password, eventname,
                        start_timestamp, end_timestamp)
                        .chain_err(|| "during Request::Event")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::User {
                   login,
                   password,
                   newlogin,
                   newpassword
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   create_user(&conn, login, password,
                        newlogin, newpassword)
                        .chain_err(|| "during Request::User")?;
                   Response::Ok(ResponseInfo::Empty)
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
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let initial_evaluation = initial_evaluation.validate()?;
                   if 0 > initial_evaluation || initial_evaluation > 10 {
                       return Err("initial_evaluation must be in range 0-10".into());
                   };
                   register_or_accept_talk(&conn, login, password,
                   speakerlogin, talk, title, start_timestamp, room,
                   initial_evaluation, eventname)
                        .chain_err(|| "during Request::Talk")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::RegisterUserForEvent {
                   login,
                   password,
                   eventname
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   register_user_for_event(&conn, login, password, eventname)
                        .chain_err(|| "during Request::RegisterUserForEvent")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::Attendance {
                   login,
                   password,
                   talk
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   attendance(&conn, login, password, talk)
                        .chain_err(|| "during Request::Attendance")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::Evaluation {
                   login,
                   password,
                   talk,
                   rating
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let rating = rating.validate()?;
                   if 0 > rating || rating > 10 {
                       return Err("rating must be in range 0-10".into());
                   };
                   evaluation(&conn, login, password, talk, rating)
                        .chain_err(|| "during Request::Evaluation")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::Reject {
                   login,
                   password,
                   talk
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   reject_spontaneous_talk(&conn, login, password, talk)
                        .chain_err(|| "during Request::Reject")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::Proposal {
                   login,
                   password,
                   talk,
                   title,
                   start_timestamp
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   propose_spontaneous_talk(&conn, login, password, talk, title, start_timestamp)
                        .chain_err(|| "during Request::Proposal")?;
                   Response::Ok(ResponseInfo::Empty)
               },

               Request::Friends {
                   login1,
                   password,
                   login2
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   make_friends(&conn, login1, password, login2)
                        .chain_err(|| "during Request::Friends")?;
                   Response::Ok(ResponseInfo::Empty)
               },

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

               Request::AttendedTalks { login, password } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let talks = attended_talks(&conn, login, password)
                        .chain_err(|| "during Request::AttendedTalks")?;
                   Response::Ok(ResponseInfo::AttendedTalks(talks))
               },

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
                   eventname
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
