use crate::name_token::{Bytes, Inscription, InscriptionMetadata, NameToken};
use bitcoin::{Block, OutPoint, Transaction, TxIn, TxOut};
use bitcoincore_rpc::RpcApi;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

const MIN_CONFIRMATIONS: u64 = 6;

struct NameTokenRepositoryState {
    next_block_height: u64,
    name_tokens: Vec<NameToken>,
}

#[derive(Clone)]
pub struct NameTokenRepository {
    state: Arc<Mutex<NameTokenRepositoryState>>,
}

impl NameTokenRepository {
    pub async fn create() -> Self {
        let initial_state = NameTokenRepositoryState {
            next_block_height: 0,
            name_tokens: Vec::new(),
        };
        let this = Self {
            state: Arc::new(Mutex::new(initial_state)),
        };
        let this_clone = this.clone();
        tokio::spawn(async move {
            this_clone.watch_blockchain().await;
        });
        this
    }

    fn bitcoin_client(&self) -> bitcoincore_rpc::Client {
        bitcoincore_rpc::Client::new(
            "http://0.0.0.0:18443",
            bitcoincore_rpc::Auth::UserPass("rpcuser".into(), "rpcpassword".into()),
        )
        .unwrap()
    }

    async fn watch_blockchain(&self) {
        loop {
            self.sync_blocks().await;
            tokio::time::sleep(Duration::from_secs(
                6, // 10 minutes
            ))
            .await;
        }
    }

    // This function would typically sync the repository state with the current state of the blockchain.
    async fn sync_blocks(&self) {
        loop {
            let state_next_blockheight = self.state.lock().unwrap().next_block_height;
            let blockchain_num_blocks = self
                .bitcoin_client()
                .get_blockchain_info()
                .expect("Failed to get blockchain info")
                .blocks;
            if state_next_blockheight >= blockchain_num_blocks - MIN_CONFIRMATIONS {
                break;
            }
            self.sync_next_block(state_next_blockheight).await;
        }
    }

    async fn sync_next_block(&self, next_blockheight: u64) {
        println!("Syncing block at height {}", next_blockheight);
        let block_hash = self
            .bitcoin_client()
            .get_block_hash(next_blockheight)
            .expect("Failed to get block hash");
        let block = self
            .bitcoin_client()
            .get_block(&block_hash)
            .expect("Failed to get block");
        self.sync_block(next_blockheight, &block).await;
    }

    async fn sync_block(&self, blockheight: u64, block: &Block) {
        let mut pending_block_updates = HashMap::new();
        for (blockindex, transaction) in block.txdata.iter().enumerate() {
            self.sync_transaction(
                transaction,
                blockindex,
                blockheight,
                &mut pending_block_updates,
            )
            .await;
        }
        self.save_block_updates(blockheight, &pending_block_updates)
            .await;
    }

    async fn sync_transaction(
        &self,
        transaction: &Transaction,
        blockindex: usize,
        blockheight: u64,
        mut pending_block_updates: &mut HashMap<OutPoint, NameToken>,
    ) {
        let num_positional_correlation =
            usize::max(transaction.input.len(), transaction.output.len());
        for positional_correlation in 0..num_positional_correlation {
            let txin = transaction.input.get(positional_correlation);
            let txout = transaction.output.get(positional_correlation);
            let metadata = InscriptionMetadata {
                txid: transaction.compute_txid(),
                vout: positional_correlation as u32,
                blockheight,
                blockindex,
            };
            self.sync_txin_txout_positional_correlation(
                txin,
                txout,
                metadata,
                &mut pending_block_updates,
            )
            .await;
        }
    }

    async fn sync_txin_txout_positional_correlation(
        &self,
        txin: Option<&TxIn>,
        txout: Option<&TxOut>,
        metadata: InscriptionMetadata,
        pending_block_updates: &mut HashMap<OutPoint, NameToken>,
    ) {
        let input_name_token = match txin {
            None => None,
            Some(txin) => {
                self.get_name_token_by_outpoint(txin.previous_output, &pending_block_updates)
                    .await
            }
        };
        let output_inscription = match txout {
            None => None,
            Some(txout) => Inscription::from_txout(txout),
        };
        let updated_name_tokens = NameToken::generate_name_token_updates(
            input_name_token.as_ref(),
            output_inscription.as_ref(),
            metadata,
        );
        for updated_name_token in updated_name_tokens {
            pending_block_updates.insert(
                updated_name_token.last_outpoint(),
                updated_name_token.clone(),
            );
        }
    }

    async fn get_name_token_by_outpoint(
        &self,
        outpoint: OutPoint,
        pending_block_updates: &HashMap<OutPoint, NameToken>,
    ) -> Option<NameToken> {
        match pending_block_updates.get(&outpoint) {
            Some(name_token) => Some(name_token.clone()),
            None => {
                let state = self.state.lock().unwrap();
                state
                    .name_tokens
                    .iter()
                    .cloned()
                    .find(|name_token| name_token.last_outpoint() == outpoint)
            }
        }
    }

    async fn save_block_updates(
        &self,
        blockheight: u64,
        pending_block_updates: &HashMap<OutPoint, NameToken>,
    ) {
        let mut state = self.state.lock().unwrap();
        for pending_name_token in pending_block_updates.values() {
            let position_to_remove = state.name_tokens.iter().position(|nt| {
                nt.first_inscription_metadata == pending_name_token.first_inscription_metadata
            });
            if let Some(position) = position_to_remove {
                state.name_tokens.remove(position);
            }
            if pending_name_token.is_revoked() {
                continue; // Don't save revoked name tokens
            }
            state.name_tokens.push(pending_name_token.clone());
        }
        state.next_block_height = blockheight + 1;
    }

    pub async fn get_name_token(&self, label: &Bytes) -> Option<NameToken> {
        let state = self.state.lock().unwrap();
        let name_token = NameToken::select_valid_name_token(label, &state.name_tokens);
        name_token.cloned()
    }
}
