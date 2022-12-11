// @generated automatically by Diesel CLI.

diesel::table! {
    questions (id) {
        id -> Int4,
        test_id -> Int4,
        text -> Text,
    }
}

diesel::table! {
    tests (id) {
        id -> Int4,
        name -> Varchar,
        description -> Nullable<Text>,
        instructions -> Nullable<Text>,
        logo -> Nullable<Varchar>,
        points_reference -> Jsonb,
        points_interpretation -> Jsonb,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        phone_number -> Nullable<Varchar>,
        channeli_id -> Int8,
        display_picture -> Nullable<Varchar>,
        created_at -> Timestamptz,
        is_admin -> Nullable<Bool>,
    }
}

diesel::joinable!(questions -> tests (test_id));

diesel::allow_tables_to_appear_in_same_query!(
    questions,
    tests,
    users,
);
