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
    light_ecs_table (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        project -> Varchar,
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
        nip -> Varchar,
        #[max_length = 255]
        vpc_id -> Nullable<Varchar>,
        #[max_length = 255]
        subnet_id -> Nullable<Varchar>,
        #[max_length = 255]
        instance_type -> Nullable<Varchar>,
        #[max_length = 255]
        internet_charge_type -> Varchar,
        #[max_length = 255]
        status -> Varchar,
        #[max_length = 255]
        os_name -> Varchar,
        #[max_length = 255]
        os_type -> Varchar,
        #[max_length = 255]
        image_id -> Varchar,
        #[max_length = 255]
        bandwidth -> Varchar,
        #[max_length = 255]
        cloud_account -> Varchar,
        is_link_server -> Bool,
        create_at -> Nullable<Timestamp>,
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
    light_ecs_table,
    lighthouse_instance,
);
