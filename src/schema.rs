// @generated automatically by Diesel CLI.

diesel::table! {
    characterrel (id) {
        id -> Uuid,
        description -> Nullable<Text>,
        character -> Nullable<Uuid>,
        story -> Nullable<Uuid>,
    }
}

diesel::table! {
    characters (id) {
        id -> Uuid,
        timeline -> Nullable<Uuid>,
        name -> Text,
        description -> Nullable<Text>,
        created -> Nullable<Int8>,
        last_edited -> Nullable<Int8>,
        creator -> Nullable<Uuid>,
        license -> Nullable<Uuid>,
        shared -> Nullable<Uuid>,
    }
}

diesel::table! {
    grouprel (id) {
        id -> Uuid,
        group_id -> Nullable<Uuid>,
        user_id -> Nullable<Uuid>,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    licenses (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        public -> Nullable<Bool>,
        content -> Nullable<Text>,
    }
}

diesel::table! {
    stories (id) {
        id -> Uuid,
        timeline -> Nullable<Uuid>,
        name -> Text,
        description -> Nullable<Text>,
        content -> Nullable<Bytea>,
        created -> Nullable<Int8>,
        last_edited -> Nullable<Int8>,
        creator -> Nullable<Uuid>,
        license -> Nullable<Uuid>,
        shared -> Nullable<Uuid>,
        renderer -> Nullable<Text>,
    }
}

diesel::table! {
    tags (id) {
        id -> Uuid,
        story -> Nullable<Uuid>,
        value -> Text,
    }
}

diesel::table! {
    timelines (id) {
        id -> Uuid,
        created -> Nullable<Int8>,
        last_edited -> Nullable<Int8>,
        creator -> Nullable<Uuid>,
        license -> Nullable<Uuid>,
        shared -> Nullable<Uuid>,
    }
}

diesel::table! {
    universe (id) {
        id -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        created -> Nullable<Int8>,
        last_edited -> Nullable<Int8>,
        creator -> Nullable<Uuid>,
        license -> Nullable<Uuid>,
        shared -> Nullable<Uuid>,
    }
}

diesel::table! {
    usergroups (id) {
        id -> Uuid,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        fname -> Text,
        lname -> Text,
        subject -> Nullable<Uuid>,
        email -> Text,
    }
}

diesel::joinable!(characterrel -> characters (character));
diesel::joinable!(characterrel -> stories (story));
diesel::joinable!(characters -> licenses (license));
diesel::joinable!(characters -> timelines (timeline));
diesel::joinable!(characters -> usergroups (shared));
diesel::joinable!(characters -> users (creator));
diesel::joinable!(grouprel -> usergroups (group_id));
diesel::joinable!(grouprel -> users (user_id));
diesel::joinable!(stories -> licenses (license));
diesel::joinable!(stories -> timelines (timeline));
diesel::joinable!(stories -> usergroups (shared));
diesel::joinable!(stories -> users (creator));
diesel::joinable!(tags -> stories (story));
diesel::joinable!(timelines -> licenses (license));
diesel::joinable!(timelines -> usergroups (shared));
diesel::joinable!(timelines -> users (creator));
diesel::joinable!(universe -> licenses (license));
diesel::joinable!(universe -> usergroups (shared));
diesel::joinable!(universe -> users (creator));

diesel::allow_tables_to_appear_in_same_query!(
    characterrel,
    characters,
    grouprel,
    licenses,
    stories,
    tags,
    timelines,
    universe,
    usergroups,
    users,
);
