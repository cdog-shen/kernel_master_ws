// @generated automatically by Diesel CLI.

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

diesel::allow_tables_to_appear_in_same_query!(
    cron_job,
    job_log,
);
