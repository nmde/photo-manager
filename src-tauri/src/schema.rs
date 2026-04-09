// @generated automatically by Diesel CLI.

diesel::table! {
    layers (id) {
        id -> Text,
        name -> Text,
        color -> Text,
    }
}

diesel::table! {
    people (id) {
        id -> Text,
        name -> Text,
        photo -> Nullable<Text>,
        category -> Text,
    }
}

diesel::table! {
    people_categories (id) {
        id -> Text,
        name -> Text,
        color -> Text,
    }
}

diesel::table! {
    photo_groups (id) {
        id -> Text,
        name -> Text,
    }
}

diesel::table! {
    photos (name) {
        name -> Text,
        asset_path -> Text,
        title -> Nullable<Text>,
        description -> Nullable<Text>,
        tags -> Nullable<Text>,
        is_duplicate -> Nullable<Integer>,
        rating -> Nullable<Integer>,
        location -> Nullable<Text>,
        thumbnail -> Nullable<Text>,
        photo_group -> Nullable<Text>,
        date -> Nullable<Text>,
        people -> Nullable<Text>,
        hide_thumbnail -> Nullable<Integer>,
        photographer -> Nullable<Text>,
    }
}

diesel::table! {
    places (id) {
        id -> Text,
        name -> Text,
        lat -> Float,
        lng -> Float,
        layer -> Text,
        category -> Text,
        shape -> Nullable<Text>,
    }
}

diesel::table! {
    settings (setting) {
        setting -> Text,
        value -> Integer,
    }
}

diesel::table! {
    shapes (id) {
        id -> Text,
        shape_type -> Text,
        points -> Text,
        layer -> Text,
        name -> Text,
    }
}

diesel::table! {
    tags (name) {
        name -> Text,
        color -> Nullable<Text>,
        prereqs -> Nullable<Text>,
        coreqs -> Nullable<Text>,
        incompatible -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    layers,
    people,
    people_categories,
    photo_groups,
    photos,
    places,
    settings,
    shapes,
    tags,
);
