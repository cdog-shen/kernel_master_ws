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
    notification_records (id) {
        id -> Int4,
        #[max_length = 32]
        channel_type -> Varchar,
        recipient -> Text,
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
        is_enabled -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
        smtp -> Nullable<Jsonb>,
        bark -> Nullable<Jsonb>,
        gotify -> Nullable<Jsonb>,
        teams_hook -> Nullable<Jsonb>,
        webhook -> Nullable<Jsonb>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    channel_configs,
    notification_aliases,
    notification_records,
    notification_templates,
);
