// @generated automatically by Diesel CLI.

diesel::table! {
    cloud_account (id) {
        id -> Int4,
        #[max_length = 255]
        provider -> Varchar,
        #[max_length = 255]
        nick_name -> Varchar,
        #[max_length = 255]
        ak -> Varchar,
        #[max_length = 255]
        sk -> Varchar,
        is_enable -> Bool,
        update_time -> Timestamp,
        #[max_length = 255]
        comment -> Varchar,
    }
}
