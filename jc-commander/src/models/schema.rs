// @generated automatically by Diesel CLI.

diesel::table! {
    cron_job (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        script -> Varchar,
        frequency -> Bigint,
        launch_at -> Datetime,
        times -> Unsigned<Integer>,
        status -> Unsigned<Tinyint>,
        params -> Text,
        create_time -> Datetime,
        update_time -> Datetime,
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
        worker -> Nullable<Varchar>,
        status -> Unsigned<Tinyint>,
        params -> Text,
        result -> Text,
        create_time -> Nullable<Datetime>,
        finish_time -> Nullable<Datetime>,
        update_time -> Nullable<Datetime>,
        #[max_length = 255]
        comment -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cron_job,
    job_log,
);
