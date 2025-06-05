// @generated automatically by Diesel CLI.

diesel::table! {
    deleted_files (id) {
        id -> Int4,
        path -> Text,
        size -> Int8,
        uploaded -> Timestamp,
        deleted -> Timestamp,
        md5 -> Text,
        sha1 -> Text,
        sha256 -> Text,
        sha512 -> Text,
        kind -> Text,
        parent -> Text,
    }
}

diesel::table! {
    files (id) {
        id -> Int4,
        path -> Text,
        size -> Int8,
        uploaded -> Timestamp,
        md5 -> Text,
        sha1 -> Text,
        sha256 -> Text,
        sha512 -> Text,
        kind -> Text,
        parent -> Text,
    }
}

diesel::table! {
    master_keys (id) {
        id -> Int4,
        value -> Text,
        created -> Timestamp,
        is_init -> Bool,
    }
}

diesel::table! {
    route_data (id) {
        id -> Int4,
        path -> Text,
        visibility -> Int2,
        created -> Timestamp,
    }
}

diesel::table! {
    token_paths (id) {
        id -> Int4,
        token -> Int4,
        path -> Text,
        added -> Timestamp,
        permission -> Int2,
    }
}

diesel::table! {
    tokens (id) {
        id -> Int4,
        name -> Text,
        value -> Text,
        created -> Timestamp,
    }
}

diesel::joinable!(token_paths -> tokens (token));

diesel::allow_tables_to_appear_in_same_query!(
    deleted_files,
    files,
    master_keys,
    route_data,
    token_paths,
    tokens,
);
