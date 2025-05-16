// @generated automatically by Diesel CLI.

diesel::table! {
    deleted_files (id) {
        id -> Integer,
        path -> Text,
        size -> BigInt,
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
        id -> Integer,
        path -> Text,
        size -> BigInt,
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
        id -> Integer,
        value -> Text,
        created -> Timestamp,
        is_init -> Bool,
    }
}

diesel::table! {
    token_paths (id) {
        id -> Integer,
        token -> Integer,
        path -> Text,
        added -> Timestamp,
        permission -> SmallInt,
    }
}

diesel::table! {
    tokens (id) {
        id -> Integer,
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
    token_paths,
    tokens,
);
