// @generated automatically by Diesel CLI.

diesel::table! {
    dns_mapping (id) {
        id -> Integer,
        tx_id -> Text,
        key -> Text,
        nostr_address -> Text,
        active -> Bool,
    }
}
