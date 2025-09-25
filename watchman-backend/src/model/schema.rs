// @generated automatically by Diesel CLI.

diesel::table! {
    access_table (id) {
        id -> Int4,
        service_id -> Int4,
        group_id -> Int4,
        group_access -> Int2,
        is_enable -> Bool,
        update_time -> Timestamp,
        #[max_length = 255]
        comment -> Varchar,
    }
}

diesel::table! {
    group_table (id) {
        id -> Int4,
        #[max_length = 255]
        group_name -> Varchar,
        is_enable -> Bool,
        user_ids -> Jsonb,
        update_time -> Timestamp,
    }
}

diesel::table! {
    service_table (id) {
        id -> Int4,
        #[max_length = 255]
        service_name -> Varchar,
        #[max_length = 255]
        nick_name -> Varchar,
        #[max_length = 255]
        service_point -> Varchar,
        is_enable -> Bool,
        update_time -> Timestamp,
    }
}

diesel::table! {
    subsystem_table (id) {
        id -> Int4,
        #[max_length = 255]
        subsys_name -> Varchar,
        #[max_length = 255]
        url -> Varchar,
        is_enable -> Bool,
        relate_service_id -> Int4,
        token -> Uuid,
        update_time -> Timestamp,
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
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        passwd -> Varchar,
        is_enable -> Bool,
        #[max_length = 255]
        full_name -> Varchar,
        contact -> Jsonb,
        last_login -> Timestamp,
        update_time -> Timestamp,
    }
}

diesel::table! {
    webhook_table (id) {
        id -> Int4,
        #[max_length = 255]
        token -> Varchar,
        #[max_length = 255]
        hook_name -> Varchar,
        #[max_length = 255]
        method_type -> Varchar,
        #[max_length = 255]
        target_url -> Varchar,
        query_json -> Jsonb,
        header_json -> Jsonb,
        body_json -> Jsonb,
        ttl -> Int8,
        is_enable -> Bool,
        update_time -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    access_table,
    group_table,
    service_table,
    subsystem_table,
    token_table,
    user_table,
    webhook_table,
);
