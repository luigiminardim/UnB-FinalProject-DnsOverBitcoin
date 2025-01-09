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
    - [x] Model Resource Records
      - [x] A;
      - [x] AAAA;
      - [x] CNAME;
      - [x] NS;
      - [x] MX;
      - [x] TXT;
  - [x] Listen to DNS requests;
  - [ ] Resolve DNS requests;
    - [x] In memory authority;
    - [ ] Zone file;
    - [ ] Catalog;
    - [ ] Recursive catalog of zones?
  - [x] Return DNS responses;
  - [x] Forward DNS requests to other DNS servers;
    - [x] Compact mode;
- [ ] Read Bitcoin UTXOs to find tokens mapping domain names to Nostr public keys;
- [ ] Fetch DNS records from Nostr network using Nostr public keys;

### Wallet

- [ ] Receive satoshis;
- [ ] Create UTXOs with tokens mapping domain names to Nostr public keys;
- [ ] Update DNS in Nostr network with new DNS records;
- [ ] Transfer Tokens to other wallets;
- [ ] Receive Tokens from other wallets;
- [ ] Revoke Tokens;
