use serde_json;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Api {
    Open {
        baza: String,
        login: String,
        password: String,
    },
}

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
