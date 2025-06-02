use crate::dns_nostr_token::DnsNostrToken;
use hickory_server::proto::rr::domain::Label;

pub struct DnsNostrTokenRepository {
    _priv: (),
}

impl DnsNostrTokenRepository {
    pub fn new() -> Self {
        DnsNostrTokenRepository { _priv: () }
    }

    /// TODO: implement a real token repository
    pub async fn get_token(&self, label: &Label) -> Option<DnsNostrToken> {
        // ## Private key
        // bech32 nsec1dlca2jrtdrv5xq7s6aku25s68sm0yqsr55a65xk8q762maw2mgwquqvqa6
        // hex    6ff1d5486b68d94303d0d76dc5521a3c36f20203a53baa1ac707b4adf5cada1c
        // ## Public key
        // bech32 npub1llme4s02jegnqnk5kudq2smj44dzv2lxztw6qky2awzxzaaa983ql8jaue
        // hex    fff79ac1ea9651304ed4b71a054372ad5a262be612dda0588aeb846177bd29e2
        let nostr_pubkey = nostr_sdk::PublicKey::from_byte_array([
            0xff, 0xf7, 0x9a, 0xc1, 0xea, 0x96, 0x51, 0x30, 0x4e, 0xd4, 0xb7, 0x1a, 0x05, 0x43,
            0x72, 0xad, 0x5a, 0x26, 0x2b, 0xe6, 0x12, 0xdd, 0xa0, 0x58, 0x8a, 0xeb, 0x84, 0x61,
            0x77, 0xbd, 0x29, 0xe2,
        ]);
        let dns_nostr_token = DnsNostrToken {
            label: label.clone(),
            nostr_pubkey,
        };
        Some(dns_nostr_token)
    }
}
