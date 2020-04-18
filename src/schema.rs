table! {
    keys (id) {
        id -> Uuid,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
        key_id -> Uuid,
        created_at -> Timestamp,
    }
}

joinable!(users -> keys (key_id));

allow_tables_to_appear_in_same_query!(keys, users,);
