use bitcoin::{block, Transaction};

use crate::name_token::{Bytes, NameToken};

pub struct NameTokenRepository {}

impl NameTokenRepository {
    pub async fn create() -> Self {
        todo!();
        // tokio::spawn(async move {

        //
        // Self {
        //     current_blockheight: 0,
        // }
    }

    // This function would typically start a background task to update the current block height.
    // For now, we will leave it unimplemented.
    async fn start(self) {
        todo!();
    }

    // This function would typically sync the repository state with the current state of the blockchain.
    async fn sync() {
        todo!();
    }

    fn sync_block(&self, blockheight: usize) {
        todo!();
    }

    fn sync_transaction(&self, transaction: Transaction) {}

    pub async fn get_name_token(&self, label: Bytes) -> Option<NameToken> {
        todo!();
    }
}
