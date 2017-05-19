use errors::*;
use serde_json;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Api {
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
}

static SECRET: &str = "d8578edf8458ce06fbc5bb76a58c5ca4";

pub fn read_call(data: &str) -> Result<Api> {
    let info: Api = serde_json::from_str(&data)?;
    Ok(info)
}


#[cfg(test)]
mod tests {
    use super::{Api, read_call, SECRET};

    #[test]
    fn deserialize_connection_info() {
        let data =
            r#"{ "open": { "baza": "stud", "login": "stud", "password": "d8578edf8458ce06fbc"}}"#;
        let info: Api = read_call(&data).unwrap();
        assert!(info ==
                Api::Open {
                    baza: "stud".to_owned(),
                    login: "stud".to_owned(),
                    password: "d8578edf8458ce06fbc".to_owned(),
                });
    }

    #[test]
    fn deserialize_organizer_info() {
        let mut data = r#"{ "organizer": { "secret": ""#.to_owned();
        data += SECRET;
        data += r#"", "newlogin": "organizer", "newpassword": "d8578edf8458ce06fbc"}}"#;
        let info: Api = read_call(&data).unwrap();
        assert!(info ==
                Api::Organizer {
                    secret: SECRET.to_owned(),
                    newlogin: "organizer".to_owned(),
                    newpassword: "d8578edf8458ce06fbc".to_owned(),
                });
    }
}
