use std::time::Duration;

pub struct NostrEventsRepository {
    nostr_relay_url: String,
}

impl NostrEventsRepository {
    pub fn new(nostr_relay_url: String) -> Self {
        NostrEventsRepository { nostr_relay_url }
    }

    pub async fn get_last_text_note_from_pubkey(
        &self,
        pubkey: nostr_sdk::PublicKey,
    ) -> Option<nostr_sdk::Event> {
        let nostr_client = nostr_sdk::Client::default();
        nostr_client.add_relay(&self.nostr_relay_url).await.unwrap();
        nostr_client.connect().await;

        let filter = nostr_sdk::Filter::new()
            .author(pubkey)
            .kind(nostr_sdk::Kind::TextNote);
        let text_notes = nostr_client
            .fetch_events(filter, Duration::from_secs(10))
            .await
            .ok()?;
        let last_text_note = text_notes.first()?;

        Some(last_text_note.clone())
    }
}
