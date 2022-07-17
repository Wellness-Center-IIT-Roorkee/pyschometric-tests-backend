table! {
    tests (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Text>,
        instructions -> Nullable<Text>,
        logo -> Nullable<Varchar>,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        phone_number -> Nullable<Varchar>,
        channeli_id -> Int8,
        display_picture -> Nullable<Varchar>,
        created_at -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(tests, users,);
