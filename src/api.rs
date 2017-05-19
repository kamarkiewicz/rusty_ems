
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Api {
    Open {
        baza: String,
        login: String,
        password: String,
    },
}


#[cfg(test)]
mod tests {
    use serde_json;
    use super::Api;

    #[test]
    fn deserialize_connection_info() {
        let data =
            r#"{ "open": { "baza": "stud", "login": "stud", "password": "d8578edf8458ce06fbc"}}"#;
        let info: Api = serde_json::from_str(data).unwrap();
        assert!(info ==
                Api::Open {
                    baza: "stud".to_owned(),
                    login: "stud".to_owned(),
                    password: "d8578edf8458ce06fbc".to_owned(),
                });
    }
}
