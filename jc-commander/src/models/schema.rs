// @generated automatically by Diesel CLI.

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
        #[max_length = 255]
        params -> Varchar,
        #[max_length = 255]
        result -> Varchar,
        create_time -> Nullable<Datetime>,
        finish_time -> Nullable<Datetime>,
        update_time -> Nullable<Datetime>,
        #[max_length = 255]
        comment -> Nullable<Varchar>,
    }
}
