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
