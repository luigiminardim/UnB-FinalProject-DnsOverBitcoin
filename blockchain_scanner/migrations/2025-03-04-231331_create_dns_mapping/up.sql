-- Your SQL goes here
CREATE TABLE "dns_mapping" (
  "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  "tx_id" TEXT NOT NULL,
  "key" TEXT NOT NULL,
  "nostr_address" TEXT NOT NULL,
  "active" BOOLEAN NOT NULL
);
