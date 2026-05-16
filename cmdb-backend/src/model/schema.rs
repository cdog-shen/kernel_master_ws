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

diesel::table! {
    job_log (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        script -> Varchar,
        #[max_length = 255]
        exec_type -> Varchar,
        #[max_length = 255]
        commander -> Varchar,
        #[max_length = 255]
        worker -> Varchar,
        status -> Int2,
        params -> Jsonb,
        result -> Text,
        finish_time -> Timestamp,
        update_time -> Timestamp,
        #[max_length = 255]
        comment -> Varchar,
    }
}

diesel::table! {
    cron_job (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        script -> Varchar,
        frequency -> Int8,
        times -> Int8,
        params -> Jsonb,
        #[max_length = 255]
        comment -> Varchar,
        is_enable -> Bool,
        launch_at -> Timestamp,
        update_time -> Timestamp,
    }
}

diesel::table! {
    cloudstorage_bucket (id) {
        id -> Int8,
        #[max_length = 255]
        provider -> Varchar,
        #[max_length = 255]
        bucket_name -> Varchar,
        #[max_length = 255]
        bucket_type -> Varchar,
        #[max_length = 255]
        location -> Varchar,
        #[max_length = 255]
        creation_date -> Varchar,
        #[max_length = 255]
        describes -> Varchar,
        tag -> Jsonb,
        full_info -> Jsonb,
        attach_info -> Jsonb,
        update_at -> Timestamp,
    }
}

// ===== watchman-backend tables (feature-gated) =====

#[cfg(feature = "full-schema")]
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

#[cfg(feature = "full-schema")]
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

#[cfg(feature = "full-schema")]
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

#[cfg(feature = "full-schema")]
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

#[cfg(feature = "full-schema")]
diesel::table! {
    token_table (tokenid) {
        #[max_length = 255]
        tokenid -> Varchar,
        #[max_length = 255]
        username -> Varchar,
        exp_time -> Timestamp,
    }
}

#[cfg(feature = "full-schema")]
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

#[cfg(feature = "full-schema")]
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

// ===== yell tables (feature-gated) =====

#[cfg(feature = "full-schema")]
diesel::table! {
    channel_configs (id) {
        id -> Int4,
        #[max_length = 32]
        channel_type -> Varchar,
        config_json -> Jsonb,
        is_enabled -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

#[cfg(feature = "full-schema")]
diesel::table! {
    notification_records (id) {
        id -> Int4,
        #[max_length = 32]
        channel_type -> Varchar,
        #[max_length = 512]
        recipient -> Varchar,
        #[max_length = 512]
        subject -> Nullable<Varchar>,
        content -> Text,
        #[max_length = 16]
        content_format -> Nullable<Varchar>,
        template_id -> Nullable<Int4>,
        #[max_length = 16]
        status -> Nullable<Varchar>,
        error_msg -> Nullable<Text>,
        sent_at -> Nullable<Timestamp>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

#[cfg(feature = "full-schema")]
diesel::table! {
    notification_templates (id) {
        id -> Int4,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 512]
        description -> Nullable<Varchar>,
        #[max_length = 32]
        channel_type -> Varchar,
        #[max_length = 512]
        subject_template -> Nullable<Varchar>,
        content_template -> Text,
        #[max_length = 16]
        content_format -> Nullable<Varchar>,
        is_enabled -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

#[cfg(feature = "full-schema")]
diesel::table! {
    notification_groups (id) {
        id -> Int4,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 512]
        description -> Nullable<Varchar>,
        is_enabled -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

#[cfg(feature = "full-schema")]
diesel::table! {
    notification_group_members (id) {
        id -> Int4,
        group_id -> Int4,
        #[max_length = 32]
        channel_type -> Varchar,
        #[max_length = 512]
        recipient -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

// ===== joinable =====

#[cfg(feature = "full-schema")]
diesel::joinable!(notification_group_members -> notification_groups (group_id));

// ===== allow_tables_to_appear_in_same_query =====

// ===== allow_tables_to_appear_in_same_query =====

#[cfg(not(feature = "full-schema"))]
diesel::allow_tables_to_appear_in_same_query!(
    cloudserver_instance,
    lighthouse_instance,
    logservice_topic,
    cloud_account,
    job_log,
    cron_job,
    cloudstorage_bucket,
);

#[cfg(feature = "full-schema")]
diesel::allow_tables_to_appear_in_same_query!(
    cloudserver_instance,
    lighthouse_instance,
    logservice_topic,
    cloud_account,
    job_log,
    cron_job,
    cloudstorage_bucket,
    access_table,
    group_table,
    service_table,
    subsystem_table,
    token_table,
    user_table,
    webhook_table,
    channel_configs,
    notification_records,
    notification_templates,
    notification_groups,
    notification_group_members,
);
