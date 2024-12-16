// @generated automatically by Diesel CLI.

diesel::table! {
    access_table (id) {
        id -> Unsigned<Integer>,
        service_id -> Unsigned<Integer>,
        group_id -> Unsigned<Integer>,
        group_access -> Unsigned<Tinyint>,
        is_enable -> Unsigned<Tinyint>,
        update_time -> Nullable<Datetime>,
        #[max_length = 255]
        comment -> Nullable<Varchar>,
    }
}

diesel::table! {
    group_table (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        name -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        date_update -> Nullable<Datetime>,
        #[max_length = 255]
        user_ids -> Varchar,
    }
}

diesel::table! {
    service_table (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        service_name -> Varchar,
        #[max_length = 255]
        service_point -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        create_time -> Nullable<Datetime>,
    }
}

diesel::table! {
    subsystem_table (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        subsys_name -> Varchar,
        #[max_length = 255]
        url -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        update_time -> Nullable<Datetime>,
        relate_service -> Nullable<Unsigned<Integer>>,
        #[max_length = 255]
        token -> Varchar,
    }
}

diesel::table! {
    token_table (tokenid) {
        #[max_length = 255]
        tokenid -> Varchar,
        #[max_length = 255]
        username -> Varchar,
        exp_time -> Timestamp,
    }
}

diesel::table! {
    user_table (id) {
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
        date_joined -> Nullable<Datetime>,
        last_login -> Nullable<Datetime>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    access_table,
    group_table,
    service_table,
    subsystem_table,
    token_table,
    user_table,
);
