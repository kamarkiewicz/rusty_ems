use errors::*;
use serde_json;

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
        start_timestamp: String,
        end_timestamp: String,
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
        start_timestamp: String,
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
        start_timestamp: String,
    },

    Friends {
        login1: String,
        password: String,
        login2: String,
    },

    UserPlan { login: String, limit: String },

    DayPlan { timestamp: String },

    BestTalks {
        start_timestamp: String,
        end_timestamp: String,
        limit: String,
        all: String,
    },

    MostPopularTalks {
        start_timestamp: String,
        end_timestamp: String,
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
        start_timestamp: String,
        end_timestamp: String,
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
        start_timestamp: String,
        end_timestamp: String,
        limit: String,
    },
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    #[allow(dead_code)]
    Ok { data: Option<serde_json::Value> },
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
    use errors::*;
    use super::{Request, read_call, SECRET};

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
        let data =
            r#"{ "most_popular_talks": {
            "start_timestamp": "2017-06-05", "end_timestamp": "2017-06-14", "limit": "42"}}"#;
        let info: Request = read_call(&data).unwrap();
        assert!(info ==
                Request::MostPopularTalks {
                    start_timestamp: "2017-06-05".to_owned(),
                    end_timestamp: "2017-06-14".to_owned(),
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
