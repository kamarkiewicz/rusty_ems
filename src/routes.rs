use errors::*;
use api::*;
use database::*;

pub struct Context {
    conn: Option<Connection>,
}

impl Context {
    pub fn new() -> Context {
        Context { conn: None }
    }
}

use std::fmt::Debug;
pub trait Route
    where Self: Debug + Sized
{
    fn route(self, _: &mut Context) -> Result<Response> {
        Ok(Response::NotImplemented)
    }
}

impl Route for OpenInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        ctx.conn = Some(establish_connection(self.login, self.password, self.baza)?);
        setup_database(ctx.conn.as_ref().unwrap())?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for OrganizerInfo {
    /// secret has been validated on Request creation
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        create_organizer_account(conn, self.newlogin, self.newpassword)
            .chain_err(|| "during Request::Organizer")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for EventInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let start_timestamp = match self.start_timestamp {
            Timestamp::Date(d) => d.and_hms(0, 0, 0),
            Timestamp::DateTime(dt) => dt,
        };
        let end_timestamp = match self.end_timestamp {
            Timestamp::Date(d) => d.and_hms(23, 59, 59),
            Timestamp::DateTime(dt) => dt,
        };
        create_event(conn,
                     self.login,
                     self.password,
                     self.eventname,
                     start_timestamp,
                     end_timestamp)
                .chain_err(|| "during Request::Event")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for UserInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        create_user(conn,
                    self.login,
                    self.password,
                    self.newlogin,
                    self.newpassword)
                .chain_err(|| "during Request::User")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for TalkInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let initial_evaluation = self.initial_evaluation.validate()?;
        if 0 > initial_evaluation || initial_evaluation > 10 {
            bail!("initial_evaluation must be in range 0-10")
        }
        register_or_accept_talk(conn,
                                self.login,
                                self.password,
                                self.speakerlogin,
                                self.talk,
                                self.title,
                                self.start_timestamp,
                                self.room,
                                initial_evaluation,
                                self.eventname)
                .chain_err(|| "during Request::Talk")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for RegisterUserForEventInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        register_user_for_event(conn, self.login, self.password, self.eventname)
            .chain_err(|| "during Request::RegisterUserForEvent")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for AttendanceInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        attendance(conn, self.login, self.password, self.talk)
            .chain_err(|| "during Request::Attendance")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for EvaluationInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let rating = self.rating.validate()?;
        if 0 > rating || rating > 10 {
            bail!("rating must be in range 0-10")
        }
        evaluation(conn, self.login, self.password, self.talk, rating)
            .chain_err(|| "during Request::Evaluation")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for RejectInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        reject_spontaneous_talk(conn, self.login, self.password, self.talk)
            .chain_err(|| "during Request::Reject")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for ProposalInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        propose_spontaneous_talk(conn,
                                 self.login,
                                 self.password,
                                 self.talk,
                                 self.title,
                                 self.start_timestamp)
                .chain_err(|| "during Request::Proposal")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for FriendsInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        make_friends(conn, self.login1, self.password, self.login2)
            .chain_err(|| "during Request::Friends")?;
        Ok(Response::Ok(ResponseInfo::Empty))
    }
}

impl Route for UserPlanInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let limit = self.limit.validate()?;
        let user_plans = user_plan(conn, self.login, limit)
            .chain_err(|| "during Request::UserPlan")?;
        Ok(Response::Ok(ResponseInfo::UserPlans(user_plans)))
    }
}

impl Route for DayPlanInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let day_plans = day_plan(conn, self.timestamp)
            .chain_err(|| "during Request::DayPlan")?;
        Ok(Response::Ok(ResponseInfo::DayPlans(day_plans)))
    }
}

impl Route for BestTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let start_timestamp = match self.start_timestamp {
            Timestamp::Date(d) => d.and_hms(0, 0, 0),
            Timestamp::DateTime(dt) => dt,
        };
        let end_timestamp = match self.end_timestamp {
            Timestamp::Date(d) => d.and_hms(23, 59, 59),
            Timestamp::DateTime(dt) => dt,
        };
        let limit: u32 = self.limit.validate()?;
        let all: bool = self.all.validate()? == 1;
        let best_talks = best_talks(conn, start_timestamp, end_timestamp, limit, all)
            .chain_err(|| "during Request::BestTalks")?;
        Ok(Response::Ok(ResponseInfo::BestTalks(best_talks)))
    }
}

impl Route for MostPopularTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let start_timestamp = match self.start_timestamp {
            Timestamp::Date(d) => d.and_hms(0, 0, 0),
            Timestamp::DateTime(dt) => dt,
        };
        let end_timestamp = match self.end_timestamp {
            Timestamp::Date(d) => d.and_hms(23, 59, 59),
            Timestamp::DateTime(dt) => dt,
        };
        let limit: u32 = self.limit.validate()?;
        let most_popular_talks = most_popular_talks(conn, start_timestamp, end_timestamp, limit)
            .chain_err(|| "during Request::MostPopularTalks")?;
        Ok(Response::Ok(ResponseInfo::MostPopularTalks(most_popular_talks)))
    }
}

impl Route for AttendedTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let talks = attended_talks(conn, self.login, self.password)
            .chain_err(|| "during Request::AttendedTalks")?;
        Ok(Response::Ok(ResponseInfo::AttendedTalks(talks)))
    }
}

impl Route for AbandonedTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let limit: u32 = self.limit.validate()?;
        let talks = abandoned_talks(conn, self.login, self.password, limit)
            .chain_err(|| "during Request::AbandonedTalks")?;
        Ok(Response::Ok(ResponseInfo::AbandonedTalks(talks)))
    }
}

impl Route for RecentlyAddedTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let limit: u32 = self.limit.validate()?;
        let talks = recently_added_talks(conn, limit)
            .chain_err(|| "during Request::RecentlyAddedTalks")?;
        Ok(Response::Ok(ResponseInfo::RecentlyAddedTalks(talks)))
    }
}

impl Route for RejectedTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let talks = rejected_talks(conn, self.login, self.password)
            .chain_err(|| "during Request::RejectedTalks")?;
        Ok(Response::Ok(ResponseInfo::RejectedTalks(talks)))
    }
}

impl Route for ProposalsInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let talks = proposals(conn, self.login, self.password)
            .chain_err(|| "during Request::Proposals")?;
        Ok(Response::Ok(ResponseInfo::Proposals(talks)))
    }
}

impl Route for FriendsTalksInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let start_timestamp = match self.start_timestamp {
            Timestamp::Date(d) => d.and_hms(0, 0, 0),
            Timestamp::DateTime(dt) => dt,
        };
        let end_timestamp = match self.end_timestamp {
            Timestamp::Date(d) => d.and_hms(23, 59, 59),
            Timestamp::DateTime(dt) => dt,
        };
        let limit: u32 = self.limit.validate()?;
        let talks = friends_talks(conn,
                                  self.login,
                                  self.password,
                                  start_timestamp,
                                  end_timestamp,
                                  limit)
                .chain_err(|| "during Request::FriendsTalks")?;
        Ok(Response::Ok(ResponseInfo::FriendsTalks(talks)))
    }
}

impl Route for FriendsEventsInfo {
    fn route(self, ctx: &mut Context) -> Result<Response> {
        let conn = ctx.conn.as_ref().ok_or("establish connection first")?;
        let talks = friends_events(conn, self.login, self.password, self.eventname)
            .chain_err(|| "during Request::FriendsEvents")?;
        Ok(Response::Ok(ResponseInfo::FriendsEvents(talks)))
    }
}

impl Route for RecommendedTalksInfo {}

impl Request {
    pub fn resolve(self, ctx: &mut Context) -> Result<Response> {
        use Request::*;
        match self {
            Open(info) => info.route(ctx),
            Organizer(info) => info.route(ctx),
            Event(info) => info.route(ctx),
            User(info) => info.route(ctx),
            Talk(info) => info.route(ctx),
            RegisterUserForEvent(info) => info.route(ctx),
            Attendance(info) => info.route(ctx),
            Evaluation(info) => info.route(ctx),
            Reject(info) => info.route(ctx),
            Proposal(info) => info.route(ctx),
            Friends(info) => info.route(ctx),
            UserPlan(info) => info.route(ctx),
            DayPlan(info) => info.route(ctx),
            BestTalks(info) => info.route(ctx),
            MostPopularTalks(info) => info.route(ctx),
            AttendedTalks(info) => info.route(ctx),
            AbandonedTalks(info) => info.route(ctx),
            RecentlyAddedTalks(info) => info.route(ctx),
            RejectedTalks(info) => info.route(ctx),
            Proposals(info) => info.route(ctx),
            FriendsTalks(info) => info.route(ctx),
            FriendsEvents(info) => info.route(ctx),
            RecommendedTalks(info) => info.route(ctx),
        }
    }
}
