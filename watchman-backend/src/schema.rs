// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(mysql_type(name = "Set"))]
    pub struct UserGroupsSet;
}

diesel::table! {
    access (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        service_point -> Varchar,
        #[max_length = 255]
        access_group -> Varchar,
        #[max_length = 255]
        group_access -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        date_update -> Nullable<Datetime>,
    }
}

diesel::table! {
    group (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        group -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        date_update -> Nullable<Datetime>,
    }
}

diesel::table! {
    token (token) {
        #[max_length = 255]
        token -> Varchar,
        #[max_length = 255]
        user -> Varchar,
        exp_time -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserGroupsSet;

    user (id) {
        id -> Unsigned<Integer>,
        #[max_length = 255]
        user -> Varchar,
        #[max_length = 255]
        passwd -> Varchar,
        is_enable -> Unsigned<Tinyint>,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        contact -> Nullable<Varchar>,
        #[max_length = 5]
        groups -> Nullable<UserGroupsSet>,
        date_joined -> Nullable<Datetime>,
        last_login -> Nullable<Datetime>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    access,
    group,
    token,
    user,
);
