// @generated automatically by Diesel CLI.

diesel::table! {
    cloudserver_instance (id) {
        id -> Int8,
        #[max_length = 255]
        provider -> Varchar,
        #[max_length = 255]
        zone -> Varchar,
        #[max_length = 255]
        instance_id -> Varchar,
        #[max_length = 255]
        instance_name -> Varchar,
        #[max_length = 255]
        platform -> Varchar,
        #[max_length = 255]
        status -> Varchar,
        #[max_length = 255]
        private_ip -> Varchar,
        tag -> Jsonb,
        full_info -> Jsonb,
        attach_info -> Jsonb,
        update_at -> Timestamp,
    }
}

diesel::table! {
    lighthouse_instance (id) {
        id -> Int8,
        #[max_length = 255]
        provider -> Varchar,
        #[max_length = 255]
        zone -> Varchar,
        #[max_length = 255]
        instance_id -> Varchar,
        #[max_length = 255]
        instance_name -> Varchar,
        #[max_length = 255]
        platform -> Varchar,
        #[max_length = 255]
        status -> Varchar,
        #[max_length = 255]
        private_ip -> Varchar,
        tag -> Jsonb,
        full_info -> Jsonb,
        attach_info -> Jsonb,
        update_at -> Timestamp,
    }
}

diesel::table! {
    logservice_topic (id) {
        id -> Int8,
        #[max_length = 255]
        provider -> Varchar,
        #[max_length = 255]
        set_id -> Varchar,
        #[max_length = 255]
        topic_id -> Varchar,
        #[max_length = 255]
        topic_name -> Varchar,
        #[max_length = 255]
        status -> Varchar,
        hot_period -> Int4,
        period -> Int4,
        index -> Bool,
        #[max_length = 255]
        describes -> Varchar,
        tag -> Jsonb,
        full_info -> Jsonb,
        attach_info -> Jsonb,
        update_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cloudserver_instance,
    lighthouse_instance,
    logservice_topic,
);
