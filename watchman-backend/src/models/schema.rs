// @generated automatically by Diesel CLI.

diesel::table! {
    access_table (id) {
        id -> Integer,
        service_id -> Unsigned<Integer>,
        group_id -> Unsigned<Integer>,
        group_access -> Unsigned<Tinyint>,
        is_enable -> Unsigned<Tinyint>,
        update_time -> Nullable<Datetime>,
    }
}

diesel::table! {
    group (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        name -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        date_update -> Nullable<Datetime>,
    }
}

diesel::table! {
    service_table (id) {
        id -> Integer,
        #[max_length = 255]
        service_name -> Varchar,
        #[max_length = 255]
        service_point -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        create_time -> Nullable<Datetime>,
    }
}

diesel::table! {
    token (tokenid) {
        #[max_length = 255]
        tokenid -> Varchar,
        #[max_length = 255]
        username -> Varchar,
        exp_time -> Timestamp,
    }
}

diesel::table! {
    user (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        passwd -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        contact -> Nullable<Varchar>,
        #[max_length = 255]
        groups -> Nullable<Varchar>,
        date_joined -> Nullable<Datetime>,
        last_login -> Nullable<Datetime>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    access_table,
    group,
    service_table,
    token,
    user,
);
