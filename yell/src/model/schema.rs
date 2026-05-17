// @generated automatically by Diesel CLI.

diesel::table! {
    channel_configs (id) {
        id -> Int4,
        #[max_length = 32]
        channel_type -> Varchar,
        config_json -> Jsonb,
        is_enabled -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        #[max_length = 128]
        name -> Varchar,
    }
}

diesel::table! {
    notification_aliases (id) {
        id -> Int4,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 512]
        description -> Nullable<Varchar>,
        recipients -> Jsonb,
        is_enabled -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

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
        params_template -> Nullable<Jsonb>,
    }
}

diesel::joinable!(notification_group_members -> notification_groups (group_id));

diesel::allow_tables_to_appear_in_same_query!(
    channel_configs,
    notification_aliases,
    notification_group_members,
    notification_groups,
    notification_records,
    notification_templates,
);
