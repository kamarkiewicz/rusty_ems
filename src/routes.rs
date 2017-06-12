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
                   setup_database(self.conn.as_ref().unwrap())?;
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
                       bail!("initial_evaluation must be in range 0-10")
                   }
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
                       bail!("rating must be in range 0-10")
                   }
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

               Request::UserPlan { login, limit } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let limit = limit.validate()?;
                   let user_plans = user_plan(&conn, login, limit)
                        .chain_err(|| "during Request::UserPlan")?;
                   Response::Ok(ResponseInfo::UserPlans(user_plans))
               },

               Request::DayPlan { timestamp } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let day_plans = day_plan(&conn, timestamp)
                        .chain_err(|| "during Request::DayPlan")?;
                   Response::Ok(ResponseInfo::DayPlans(day_plans))
               },

               Request::BestTalks {
                   start_timestamp,
                   end_timestamp,
                   limit,
                   all
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
                   let limit: u32 = limit.validate()?;
                   let all: bool = all.validate()? == 1;
                   let best_talks = best_talks(&conn, start_timestamp, end_timestamp, limit, all)
                        .chain_err(|| "during Request::BestTalks")?;
                   Response::Ok(ResponseInfo::BestTalks(best_talks))
               },

               Request::MostPopularTalks {
                   start_timestamp,
                   end_timestamp,
                   limit
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
                   let limit: u32 = limit.validate()?;
                   let most_popular_talks = most_popular_talks(&conn,
                            start_timestamp, end_timestamp, limit)
                        .chain_err(|| "during Request::MostPopularTalks")?;
                   Response::Ok(ResponseInfo::MostPopularTalks(most_popular_talks))
               },

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
               } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let limit: u32 = limit.validate()?;
                   let talks = abandoned_talks(&conn, login, password, limit)
                        .chain_err(|| "during Request::AbandonedTalks")?;
                   Response::Ok(ResponseInfo::AbandonedTalks(talks))
               },

               Request::RecentlyAddedTalks { limit } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let limit: u32 = limit.validate()?;
                   let talks = recently_added_talks(&conn, limit)
                        .chain_err(|| "during Request::RecentlyAddedTalks")?;
                   Response::Ok(ResponseInfo::RecentlyAddedTalks(talks))
               },

               Request::RejectedTalks { login, password } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let talks = rejected_talks(&conn, login, password)
                        .chain_err(|| "during Request::RejectedTalks")?;
                   Response::Ok(ResponseInfo::RejectedTalks(talks))
               },

               Request::Proposals { login, password } => {
                   let conn = self.conn.as_ref().ok_or("establish connection first")?;
                   let talks = proposals(&conn, login, password)
                        .chain_err(|| "during Request::Proposals")?;
                   Response::Ok(ResponseInfo::Proposals(talks))
               },

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
