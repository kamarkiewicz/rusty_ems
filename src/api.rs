use errors::*;
use serde_json;

pub use self::timestamp_fmt::Timestamp;

/// Timestamp formatter used by Serde. Supports ISO 8601
/// https://www.postgresql.org/docs/current/static/datatype-datetime.html
mod timestamp_fmt {
    pub use chrono::NaiveDateTime as Timestamp;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error> where S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<D>(D) -> Result<T, D::Error> where D: Deserializer
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Timestamp::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}


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
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        room: String,
        initial_evaluation: String,
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
        rating: String,
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
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
    },

    Friends {
        login1: String,
        password: String,
        login2: String,
    },

    UserPlan { login: String, limit: String },

    DayPlan {
        #[serde(with = "timestamp_fmt")]
        timestamp: Timestamp,
    },

    BestTalks {
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: String,
        all: String,
    },

    MostPopularTalks {
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: String,
    },

    AttendedTalks { login: String, password: String },

    AbandonedTalks {
        login: String,
        password: String,
        limit: String,
    },

    RecentlyAddedTalks { limit: String },

    RejectedTalks { login: String, password: String },

    Proposals { login: String, password: String },

    FriendsTalks {
        login: String,
        password: String,
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: String,
    },

    FriendsEvents {
        login: String,
        password: String,
        event: String,
    },

    RecommendedTalks {
        login: String,
        password: String,
        #[serde(with = "timestamp_fmt")]
        start_timestamp: Timestamp,
        #[serde(with = "timestamp_fmt")]
        end_timestamp: Timestamp,
        limit: String,
    },
}

pub enum Response {
    #[allow(dead_code)]
    Ok(Option<String>),
    NotImplemented,
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
        let info: Request = read_call(&data).unwrap();

        let common_timestamp =
            Timestamp::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S").unwrap();
        assert!(info ==
                Request::MostPopularTalks {
                    start_timestamp: common_timestamp,
                    end_timestamp: common_timestamp,
                    limit: "42".to_owned(),
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
}
