table! {
    events (event_id) {
        event_id -> Int8,
        eventname -> Text,
        start_timestamp -> Timestamp,
        end_timestamp -> Timestamp,
    }
}

table! {
    talks (talk_id) {
        talk_id -> Text,
        status -> Int2,
        title -> Text,
        user_id_User -> Nullable<Int8>,
        event_id_Event -> Nullable<Int8>,
        start_timestamp -> Timestamp,
        add_timestamp -> Timestamp,
    }
}

table! {
    users (user_id) {
        user_id -> Int8,
        login -> Varchar,
        password -> Text,
        is_organizer -> Bool,
    }
}

table! {
    User_attended_for_Talk (user_id_User, talk_id_Talk) {
        user_id_User -> Int8,
        talk_id_Talk -> Text,
        present -> Bool,
        rating -> Nullable<Int2>,
    }
}

table! {
    User_friend_of_User (user1_id, user2_id) {
        user1_id -> Int8,
        user2_id -> Int8,
    }
}

table! {
    User_registered_for_Event (user_id_User, event_id_Event) {
        user_id_User -> Int8,
        event_id_Event -> Int8,
    }
}
