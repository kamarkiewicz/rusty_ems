use errors::*;
use serde_json;

pub use chrono::NaiveDateTime as DateTime;
pub use chrono::NaiveDate as Date;

#[derive(Debug, PartialEq)]
pub enum Timestamp {
    Date(Date),
    DateTime(DateTime),
}

/// Timestamp formatter used by Serde. Supports ISO 8601
/// https://www.postgresql.org/docs/current/static/datatype-datetime.html
mod timestamp_fmt {
    use super::{Timestamp, Date, DateTime};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT_DATETIME: &'static str = "%Y-%m-%d %H:%M:%S";
    const FORMAT_DATE: &'static str = "%Y-%m-%d";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let datetime = DateTime::parse_from_str(&s, FORMAT_DATETIME)
            .map(|dt| Timestamp::DateTime(dt));
        let date = |_| Date::parse_from_str(&s, FORMAT_DATE).map(|d| Timestamp::Date(d));
        datetime.or_else(date).map_err(serde::de::Error::custom)
    }
}

mod date_fmt {
    use super::Date;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Date::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod datetime_fmt {
    use super::DateTime;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StrOr<T> {
    Str(String),
    Typ(T),
}

use std::str::FromStr;
use std::num::ParseIntError;
impl<T> StrOr<T>
    where T: FromStr<Err = ParseIntError>
{
    pub fn validate(self) -> Result<T> {
        use StrOr::*;
        match self {
            Str(s) => FromStr::from_str(&s[..]).map_err(|e: ParseIntError| e.into()),
            Typ(t) => Ok(t),
        }
    }
}

// ================================================
//                    REQUESTS
// ================================================

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Request {
    Open {
        baza: String,
        login: String,
        password: String,
    },

    Organizer {
        secret: String,
        newlogin: String,
        newpassword: String,
    },

    Event {
        login: String,
        password: String,
        eventname: String,
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
    },

    User {
        login: String,
        password: String,
        newlogin: String,
        newpassword: String,
    },

    Talk {
        login: String,
        password: String,
        speakerlogin: String,
        talk: String,
        title: String,
        #[serde(with = "datetime_fmt")]
        start_timestamp: DateTime,
        room: String,
        initial_evaluation: StrOr<i16>,
        eventname: String,
    },

    RegisterUserForEvent {
        login: String,
        password: String,
        eventname: String,
    },

    Attendance {
        login: String,
        password: String,
        talk: String,
    },

    Evaluation {
        login: String,
        password: String,
        talk: String,
        rating: StrOr<i16>,
    },

    Reject {
        login: String,
        password: String,
        talk: String,
    },

    Proposal {
        login: String,
        password: String,
        talk: String,
        title: String,
        #[serde(with = "datetime_fmt")]
        start_timestamp: DateTime,
    },

    Friends {
        login1: String,
        password: String,
        login2: String,
    },

    UserPlan { login: String, limit: StrOr<u32> },

    DayPlan {
        #[serde(with = "date_fmt")]
        timestamp: Date,
    },

    BestTalks {
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: StrOr<u32>,
        all: StrOr<u32>,
    },

    MostPopularTalks {
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: StrOr<u32>,
    },

    AttendedTalks { login: String, password: String },

    AbandonedTalks {
        login: String,
        password: String,
        limit: StrOr<u32>,
    },

    RecentlyAddedTalks { limit: StrOr<u32> },

    RejectedTalks { login: String, password: String },

    Proposals { login: String, password: String },

    FriendsTalks {
        login: String,
        password: String,
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: StrOr<u32>,
    },

    FriendsEvents {
        login: String,
        password: String,
        eventname: String,
    },

    RecommendedTalks {
        login: String,
        password: String,
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: StrOr<u32>,
    },
}

// ================================================
//                    RESPONSES
// ================================================

pub enum Response {
    #[allow(dead_code)]
    Ok(ResponseInfo),
    NotImplemented,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ResponseInfo {
    AttendedTalks(Vec<AttendedTalk>),
    UserPlans(Vec<UserPlan>),
    DayPlans(Vec<DayPlan>),
    BestTalks(Vec<BestTalk>),
    MostPopularTalks(Vec<MostPopularTalk>),
    AbandonedTalks(Vec<AbandonedTalk>),
    RecentlyAddedTalks(Vec<RecentlyAddedTalk>),
    RejectedTalks(Vec<RejectedTalk>),
    Proposals(Vec<Proposal>),
    FriendsTalks(Vec<FriendsTalk>),
    FriendsEvents(Vec<FriendsEvent>),
    RecommendedTalks(Vec<RecommendedTalk>),
    Empty,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AttendedTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct UserPlan {
    pub login: String,
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DayPlan {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct BestTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct MostPopularTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AbandonedTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: i64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RecentlyAddedTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: u32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RejectedTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: u32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Proposal {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: u32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FriendsTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: u32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FriendsEvent {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: u32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RecommendedTalk {
    pub talk: String,
    #[serde(with = "datetime_fmt")]
    pub start_timestamp: DateTime,
    pub title: String,
    pub room: String,
    pub number: u32,
}


static SECRET: &str = "d8578edf8458ce06fbc5bb76a58c5ca4";

pub fn read_call(data: &str) -> Result<Request> {
    let info: Request = serde_json::from_str(data)?;
    match &info {
        &Request::Organizer { ref secret, .. } => {
            if secret != SECRET {
                bail!("invalid secret")
            }
        }
        _ => {}
    }
    Ok(info)
}


// ================================================
//                    TESTS
// ================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_connection_info() {
        let data =
            r#"{ "open": { "baza": "stud", "login": "stud", "password": "d8578edf8458ce06fbc"}}"#;
        let info: Request = read_call(&data).unwrap();
        assert!(info ==
                Request::Open {
                    baza: "stud".to_owned(),
                    login: "stud".to_owned(),
                    password: "d8578edf8458ce06fbc".to_owned(),
                });
    }

    #[test]
    fn deserialize_organizer_info_with_valid_secret() {
        let mut data = r#"{ "organizer": { "secret": ""#.to_owned();
        data += SECRET;
        data += r#"", "newlogin": "organizer", "newpassword": "d8578edf8458ce06fbc"}}"#;
        let info: Request = read_call(&data).unwrap();
        assert!(info ==
                Request::Organizer {
                    secret: SECRET.to_owned(),
                    newlogin: "organizer".to_owned(),
                    newpassword: "d8578edf8458ce06fbc".to_owned(),
                });
    }

    #[test]
    fn deserialize_snake_case() {
        let data = r#"{ "most_popular_talks": {
            "start_timestamp": "2015-09-05 23:56:04",
            "end_timestamp": "2015-09-05 23:56:04",
            "limit": "42"}}"#;
        let info: Request = read_call(&data).expect("json input needs a fix");

        let timestamp = DateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S")
            .expect("not a timestamp");
        let start_timestamp = Timestamp::DateTime(timestamp);
        let end_timestamp = Timestamp::DateTime(timestamp);

        assert!(info ==
                Request::MostPopularTalks {
                    start_timestamp: start_timestamp,
                    end_timestamp: end_timestamp,
                    limit: StrOr::Str("42".to_owned()),
                });
    }

    #[test]
    fn deserialize_organizer_info_with_invalid_secret() {
        let mut data = r#"{ "organizer": { "secret": ""#.to_owned();
        data += "INVALIDSECRET";
        data += r#"", "newlogin": "organizer", "newpassword": "d8578edf8458ce06fbc"}}"#;
        let err = read_call(&data).unwrap_err();
        match err {
            Error(ErrorKind::Msg(msg), ..) => assert!(msg == "invalid secret"),
            _ => assert!(false),
        }
    }

    #[test]
    fn deserialize_event_tests_timestamps() {
        let data = r#"{"event": {"login": "Donald_Grump11", "password": "admin",
                       "eventname": "Konwent",
                       "start_timestamp": "2016-01-20 10:00:00",
                       "end_timestamp": "2016-02-01 18:00:00"}}"#;
        let info: Request = read_call(&data).expect("json input needs a fix");

        let timestamp = DateTime::parse_from_str("2016-01-20 10:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("not a timestamp");
        let start_timestamp = Timestamp::DateTime(timestamp);
        let timestamp = DateTime::parse_from_str("2016-02-01 18:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("not a timestamp");
        let end_timestamp = Timestamp::DateTime(timestamp);

        assert!(info ==
                Request::Event {
                    login: "Donald_Grump11".to_owned(),
                    password: "admin".to_owned(),
                    eventname: "Konwent".to_owned(),
                    start_timestamp: start_timestamp,
                    end_timestamp: end_timestamp,
                });
    }
}
