// @generated automatically by Diesel CLI.

diesel::table! {
    ai_providers (id) {
        id -> Nullable<Integer>,
        name -> Text,
        provider_type -> Text,
        api_key -> Nullable<Text>,
        endpoint_url -> Nullable<Text>,
        model_name -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        provider_category -> Text,
    }
}

diesel::table! {
    content_items (id) {
        id -> Nullable<Integer>,
        title -> Text,
        description -> Nullable<Text>,
        content_type -> Text,
        content_path -> Text,
        adapter_id -> Nullable<Integer>,
        duration_minutes -> Nullable<Integer>,
        tags -> Nullable<Text>,
        node_accessibility -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        transformer_scripts -> Nullable<Text>,
        is_dj_accessible -> Bool,
    }
}

diesel::table! {
    dj_memories (id) {
        id -> Nullable<Integer>,
        dj_id -> Integer,
        memory_type -> Text,
        content -> Text,
        importance_score -> Integer,
        happened_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    dj_profiles (id) {
        id -> Nullable<Integer>,
        name -> Text,
        personality_prompt -> Text,
        voice_config_json -> Text,
        context_depth -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        voice_provider_id -> Nullable<Integer>,
        llm_provider_id -> Nullable<Integer>,
        context_script_ids -> Nullable<Text>,
        talkativeness -> Float,
    }
}

diesel::table! {
    global_settings (id) {
        id -> Nullable<Integer>,
        key -> Text,
        value -> Text,
        description -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    node_schedules (id) {
        id -> Nullable<Integer>,
        node_id -> Integer,
        schedule_id -> Integer,
        created_at -> Timestamp,
        priority -> Nullable<Integer>,
    }
}

diesel::table! {
    nodes (id) {
        id -> Nullable<Integer>,
        name -> Text,
        secret_key -> Text,
        ip_address -> Nullable<Text>,
        status -> Text,
        last_heartbeat -> Nullable<Timestamp>,
        available_paths -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        current_content_id -> Nullable<Integer>,
        playback_position_secs -> Nullable<Float>,
        playback_duration_secs -> Nullable<Float>,
        script_context -> Nullable<Text>,
    }
}

diesel::table! {
    permissions (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        resource_type -> Text,
        resource_id -> Integer,
        permission_level -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    schedule_blocks (id) {
        id -> Nullable<Integer>,
        schedule_id -> Integer,
        content_id -> Nullable<Integer>,
        day_of_week -> Nullable<Integer>,
        specific_date -> Nullable<Date>,
        start_time -> Time,
        duration_minutes -> Integer,
        script_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        dj_id -> Nullable<Integer>,
    }
}

diesel::table! {
    schedules (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        schedule_type -> Text,
        priority -> Integer,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        dj_id -> Nullable<Integer>,
    }
}

diesel::table! {
    scripts (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        script_type -> Text,
        script_content -> Text,
        parameters_schema -> Nullable<Text>,
        is_builtin -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Nullable<Integer>,
        username -> Text,
        password_hash -> Text,
        role -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(content_items -> scripts (adapter_id));
diesel::joinable!(dj_memories -> dj_profiles (dj_id));
diesel::joinable!(node_schedules -> nodes (node_id));
diesel::joinable!(node_schedules -> schedules (schedule_id));
diesel::joinable!(nodes -> content_items (current_content_id));
diesel::joinable!(permissions -> users (user_id));
diesel::joinable!(schedule_blocks -> content_items (content_id));
diesel::joinable!(schedule_blocks -> dj_profiles (dj_id));
diesel::joinable!(schedule_blocks -> schedules (schedule_id));
diesel::joinable!(schedule_blocks -> scripts (script_id));
diesel::joinable!(schedules -> dj_profiles (dj_id));

diesel::allow_tables_to_appear_in_same_query!(
    ai_providers,
    content_items,
    dj_memories,
    dj_profiles,
    global_settings,
    node_schedules,
    nodes,
    permissions,
    schedule_blocks,
    schedules,
    scripts,
    users,
);
