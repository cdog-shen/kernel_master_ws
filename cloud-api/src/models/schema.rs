// @generated automatically by Diesel CLI.

diesel::table! {
    cloud_account (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        cloud_provider -> Varchar,
        #[max_length = 255]
        nick_name -> Varchar,
        #[max_length = 255]
        ak -> Varchar,
        #[max_length = 255]
        sk -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        update_time -> Nullable<Datetime>,
        #[max_length = 255]
        comment -> Nullable<Varchar>,
    }
}
