use bdk::bitcoin::{Address, Network};
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::bitcoin::util::bip32::{DerivationPath, KeySource};
use bdk::bitcoin::Amount;
use bdk::bitcoin::consensus::encode::serialize;

use bdk::bitcoincore_rpc::{Auth as rpc_auth, Client, RpcApi};

use bdk::blockchain::rpc::{Auth, RpcBlockchain, RpcConfig, RpcSyncParams};
use bdk::blockchain::{ConfigurableBlockchain};

use bdk::keys::bip39::{Mnemonic, Language, WordCount};
use bdk::keys::{GeneratedKey, GeneratableKey, ExtendedKey, DerivableKey, DescriptorKey};
use bdk::keys::DescriptorKey::Secret;

use bdk::miniscript::miniscript::Segwitv0;

use bdk::{SyncOptions, Wallet};
use bdk::wallet::{AddressIndex, signer::SignOptions, wallet_name_from_descriptor};

use bdk::sled::{self, Tree};

use std::str::FromStr;

use hex::encode;

use std::env;
use dotenvy::dotenv;

fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Get receive and change descriptors

    let new_receive_desc;
    match env::var("RECEIVE_DESC") {
        Ok(receive_desc) => new_receive_desc = receive_desc,
        Err(_) => (new_receive_desc, _) = get_descriptors(),
    }

    let new_change_desc;
    match env::var("CHANGE_DESC") {
        Ok(change_desc) => new_change_desc = change_desc,
        Err(_) => (_, new_change_desc) = get_descriptors(),
    }

    // Use deterministic wallet name derived from descriptor
    let bdk_wallet_name = wallet_name_from_descriptor(&new_receive_desc, Some(&new_change_desc), Network::Regtest, &Secp256k1::new()).unwrap();

    // Create the datadir to store wallet data
    let mut datadir = dirs_next::home_dir().unwrap();
    datadir.push(".bdk-db");
    let database = sled::open(datadir).unwrap();
    let db_tree = database.open_tree(bdk_wallet_name.clone()).unwrap();

    // Set RPC username, password and url
    let rpc_user = env::var("RPC_USER").unwrap();
    let rpc_password = env::var("RPC_PASSWORD").unwrap();
    let auth = Auth::UserPass { username: rpc_user.clone(), password: rpc_password.clone() };
    let rpc_url = env::var("RPC_URL").unwrap();

    // Setup the RPC configuration
    let rpc_config = RpcConfig {
        url: rpc_url,
        auth,
        wallet_name: bdk_wallet_name,
        sync_params: Some(RpcSyncParams::default()),
        network: Network::Regtest,
    };

    // Use the above configuration to create a RPC blockchain backend
    let blockchain = RpcBlockchain::from_config(&rpc_config).unwrap();

    // Combine everything and finally create the BDK wallet structure
    let bdk_wallet = Wallet::new(&new_receive_desc, Some(&new_change_desc), Network::Regtest, db_tree).unwrap();

    // Fetch a fresh address to receive coins
    let bdk_address = bdk_wallet.get_address(AddressIndex::New).unwrap().address;

    // Create a RPC interface
    let rpc_auth = rpc_auth::UserPass(
        rpc_user.clone(),
        rpc_password.clone()
    );
    let url = env::var("RPC_URL").unwrap();
    let rpc_url = format!("{}/wallet/miner", url);
    let miner_rpc = Client::new(&rpc_url, rpc_auth).unwrap();

    // Send some coins from the miner to the BDK wallet
    send_from_miner_and_confirm(&miner_rpc, &bdk_address, 10000);

    // Sync the wallet with the blockchain
    bdk_wallet.sync(&blockchain, SyncOptions::default()).unwrap();

    // Get miner address
    let miner_addr = miner_rpc.get_new_address(None, None).unwrap();

    // Send some coins from BDK wallet to miner address
    send_from_bdk_wallet_and_confirm(&bdk_wallet, &miner_rpc, &miner_addr, 5000);

    // Sync the wallet with the blockchain
    bdk_wallet.sync(&blockchain, SyncOptions::default()).unwrap();

    // Get the balance of the miner
    let miner_balance = miner_rpc.get_balance(None, None).unwrap();
    println!("Miner balance: {:#?}", miner_balance);

    // Get the balance of the wallet
    let balance = bdk_wallet.get_balance().unwrap();
    println!("Wallet balance: {:#?}", balance);
}

fn send_from_miner_and_confirm(miner_rpc: &Client, dest_address: &Address, amount: u64) {
    // Send sats from Miner to BDK
    miner_rpc.send_to_address(dest_address, Amount::from_sat(amount), None, None, None, None, None, None).unwrap();

    confirm_transaction(miner_rpc);
}

fn send_from_bdk_wallet_and_confirm(bdk_wallet: &Wallet<Tree>, miner_rpc: &Client, dest_address: &Address, amount: u64) {
    // Create a transaction builder
    let mut tx_builder = bdk_wallet.build_tx();

    // Set recipient of the transaction
    // TODO: Instead of using script pubkey, we should use dns script
    tx_builder.set_recipients(vec!((dest_address.script_pubkey(), amount)));

    // Finalize the transaction and extract the PSBT
    let (mut psbt, _) = tx_builder.finish().unwrap();

    // Set signing option
    let signopt = SignOptions {
        assume_height: None,
        ..Default::default()
    };

    // Sign the above PSBT with signing option
    bdk_wallet.sign(&mut psbt, signopt).unwrap();

    // Extract the final transaction
    let tx = psbt.extract_tx();

    // Broadcast the transaction
    let tx_hex = encode(serialize(&tx));
    miner_rpc.send_raw_transaction(tx_hex).unwrap();

    confirm_transaction(miner_rpc);
}

fn confirm_transaction(miner_rpc: &Client) {
    // Get a new address
    let miner_addr = miner_rpc.get_new_address(None, None).unwrap();

    // Confirm transaction by generating some blocks
    miner_rpc.generate_to_address(1, &miner_addr).unwrap();
}

// generate fresh descriptor strings and return them via (receive, change) tuple
fn get_descriptors() -> (String, String) {
    // Create a new secp context
    let secp = Secp256k1::new();

    // You can also set a password to unlock the mnemonic
    let desc_password = env::var("DESCRIPTOR_PASSWORD").unwrap();
    let password = Some(desc_password);

    // Generate a fresh mnemonic, and from there a privatekey
    let mnemonic: GeneratedKey<_, Segwitv0> = Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
    let mnemonic = mnemonic.into_key();
    let xkey: ExtendedKey = (mnemonic, password).into_extended_key().unwrap();
    let xprv = xkey.into_xprv(Network::Regtest).unwrap();

    // Create derived privkey from the above master privkey
    // We use the following derivation paths for receive and change keys
    // receive: "m/84h/1h/0h/0"
    // change: "m/84h/1h/0h/1"
    let mut keys = Vec::new();

    for path in ["m/84h/1h/0h/0", "m/84h/1h/0h/1"] {
        let deriv_path: DerivationPath = DerivationPath::from_str(path).unwrap();
        let derived_xprv = &xprv.derive_priv(&secp, &deriv_path).unwrap();
        let origin: KeySource = (xprv.fingerprint(&secp), deriv_path);
        let derived_xprv_desc_key: DescriptorKey<Segwitv0> = derived_xprv.into_descriptor_key(Some(origin), DerivationPath::default()).unwrap();

        // Wrap the derived key in the wpkh() string to produce a descriptor string
        if let Secret(key, _, _) = derived_xprv_desc_key {
            let mut desc = "wpkh(".to_string();
            desc.push_str(&key.to_string());
            desc.push_str(")");
            keys.push(desc);
        }
    }

    // Return the keys as a tuple
    (keys[0].clone(), keys[1].clone())
}