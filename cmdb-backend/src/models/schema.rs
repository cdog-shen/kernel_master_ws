// @generated automatically by Diesel CLI.

diesel::table! {
    cloudserver_instance (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        provider -> Varchar,
        #[max_length = 255]
        zone -> Varchar,
        #[max_length = 255]
        instance_id -> Varchar,
        #[max_length = 255]
        instance_name -> Varchar,
        #[max_length = 255]
        plantform -> Varchar,
        #[max_length = 255]
        status -> Varchar,
        #[max_length = 255]
        tag -> Varchar,
        #[max_length = 255]
        private_ip -> Varchar,
        full_info -> Nullable<Text>,
        attach_info -> Nullable<Text>,
        update_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    lighthouse_instance (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        cloud_name -> Varchar,
        #[max_length = 255]
        region -> Varchar,
        #[max_length = 255]
        zone -> Varchar,
        #[max_length = 255]
        instance_id -> Varchar,
        #[max_length = 255]
        instance_name -> Varchar,
        #[max_length = 255]
        wip -> Nullable<Varchar>,
        #[max_length = 255]
        vpc_id -> Nullable<Varchar>,
        #[max_length = 255]
        status -> Varchar,
        #[max_length = 255]
        os_name -> Varchar,
        #[max_length = 255]
        os_type -> Varchar,
        create_at -> Nullable<Timestamp>,
        update_at -> Nullable<Timestamp>,
        full_info -> Nullable<Text>,
        attach_info -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cloudserver_instance,
    lighthouse_instance,
);
