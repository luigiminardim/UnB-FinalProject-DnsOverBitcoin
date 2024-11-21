# DNS over Nostr

This project aims to create a DNS server that uses tokenized pointers in Bitcoin
blockchain to point to an Nostr public key. This way, the DNS server can resolve
domain names to Nostr public keys, which can be used to fetch the corresponding
DNS records from the Nostr network.

## Use cases

### DNS Server

- [ ] Recursive resolve or forward non-Nostr domain names using traditional DNS;
  - [ ] Model the DNS core logic;
    - [x] Model Name;
    - [x] Model Type;
    - [x] Model Resource Record;
    - [x] Model Class;
    - [ ] Model Resource Records
      - [x] A;
      - [ ] AAAA;
      - [ ] CNAME;
      - [ ] NS;
      - [ ] MX;
      - [ ] TXT;
  - [ ] Listen to DNS requests;
  - [ ] Resolve DNS requests;
  - [ ] Return DNS responses;
  - [ ] Forward DNS requests to other DNS servers;
- [ ] Read Bitcoin UTXOs to find tokens mapping domain names to Nostr public keys;
- [ ] Fetch DNS records from Nostr network using Nostr public keys;
- [ ] Recursive resolve between Nostr domain names and Traditional domain names;

### Wallet

- [ ] Receive satoshis;
- [ ] Create UTXOs with tokens mapping domain names to Nostr public keys;
- [ ] Update DNS in Nostr network with new DNS records;
- [ ] Transfer Tokens to other wallets;
- [ ] Receive Tokens from other wallets;
- [ ] Revoke Tokens;
