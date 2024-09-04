use dotenvy::from_read;
use lazy_static::lazy_static;
use std::env;
use std::io::Cursor;

// use folder_transfer::FolderTransfer;
use kinode_process_lib::http::bind_ws_path;
use kinode_process_lib::{
    await_message, call_init,
    eth::{EthConfigAction, NodeOrRpcUrl, ProviderConfig},
    http::{self},
    println, Address, Message, Request, spawn,
    OnExit, our_capabilities
};
use std::sync::RwLock;

mod contract_utils;
use contract_utils::{encrypt_data, decrypt_data, Caller, GamelordCaller};


use alloy::signers::{local::PrivateKeySigner, SignerSync};
use alloy_primitives::{Signature, U256};
use alloy_signer::{LocalWallet, Signer};

mod gamelord_types;
use gamelord_types::{
    Action, ActivePlayer, GamelordRequestMinecraft, GamelordResponseMinecraft,
    PrivateKey, SerializableWallet, State,
};
use mcstructs::{
    ChatMessage, GameLobbyDiff, McClientToGamelordRequest, Player, TeamName, WorkerRequest, WorkerStatus,
    get_worker_address, clear_worker_address,
};

mod handlers;
use handlers::{handle_driver_message, handle_http_request, handle_mcclient_request};


wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v0",
});

// spawns a worker process for folder transfer (whether it will be for receiving or sending)
fn initialize_worker(
    our: Address,
    current_worker_address: &mut Option<Address>,
) -> anyhow::Result<()> {
    let our_worker = spawn(
        None,
        &format!("{}/pkg/worker.wasm", our.package_id()),
        OnExit::None,
        our_capabilities(),
        vec![],
        false,
    )?;
    //
    // temporarily stores worker address while the worker is alive
    *current_worker_address = Some(Address {
        node: our.node.clone(),
        process: our_worker.clone(),
    });
    println!("worker address: {:?}", current_worker_address);
    println!("worker initialized");
    Ok(())
}

lazy_static! {
    pub static ref CURRENT_CHAIN_ID: u64 = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        env::var("VITE_CURRENT_CHAIN_ID").expect("CHAIN_ID must be set").parse().unwrap()
    };

    pub static ref CONTRACT_ADDRESS: String = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        match *CURRENT_CHAIN_ID {
            31337 => env::var("VITE_ANVIL_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            11155111 => env::var("VITE_SEPOLIA_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            1 => env::var("VITE_MAINNET_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            10 => env::var("VITE_OPTIMISM_CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set"),
            _ => panic!("Invalid CURRENT_CHAIN_ID: {}", *CURRENT_CHAIN_ID),
        }
    };

    pub static ref RPC_URL: NodeOrRpcUrl = {
        let env_content = include_str!("../../../.env");
        from_read(Cursor::new(env_content)).expect("Failed to parse .env content");
        match *CURRENT_CHAIN_ID {
            31337 => NodeOrRpcUrl::RpcUrl(env::var("VITE_ANVIL_RPC_URL").expect("RPC_URL must be set")),
            11155111 => NodeOrRpcUrl::RpcUrl(env::var("VITE_SEPOLIA_RPC_URL").expect("RPC_URL must be set")),
            1 => NodeOrRpcUrl::RpcUrl(env::var("VITE_MAINNET_RPC_URL").expect("RPC_URL must be set")),
            10 => NodeOrRpcUrl::RpcUrl(env::var("VITE_OPTIMISM_RPC_URL").expect("RPC_URL must be set")),
            _ => panic!("Invalid CURRENT_CHAIN_ID: {}", *CURRENT_CHAIN_ID),
        }
    };

    pub static ref MIN_ETH_WAGER: U256 = {
        "30000000000000".parse().unwrap() // 0.00003 eth
    };
}
// worker address for folder transfer
lazy_static! {  
    pub static ref WORKER_ADDRESS: RwLock<Option<Address>> = RwLock::new(None);
}


// used for eth contract testing
fn handle_terminal_message(
    state: &mut State,
    gamelord_caller: &mut Option<GamelordCaller>,
    ws_channel_id: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    println!("terminal message received");

    let action = match serde_json::from_slice::<Action>(&message.body()) {
        Ok(deserialized) => deserialized,
        Err(e) => {
            println!("Failed to deserialize message body: {:?}", e);
            return Ok(());
        }
    };

    match action {
        Action::GetPlayerInfo(funding_address) => {
            if let Some(caller) = gamelord_caller {
                let result = caller.get_player_info(funding_address);
                println!("result: {:?}", result);
            } else {
                println!("No contract caller found");
            }
        }
        Action::EncryptWallet {
            private_key,
            password,
        } => {
            let private_key = match private_key {
                Some(private_key) => private_key,
                None => {
                    if let Some(PrivateKey::Decrypted(wallet)) =
                        state.wallets.get(&CURRENT_CHAIN_ID)
                    {
                        wallet.private_key.clone()
                    } else {
                        return Err(anyhow::anyhow!("Private key already encrypted."));
                    }
                }
            };
            let encrypted_wallet_data = encrypt_data(private_key.as_bytes(), password.as_str());
            state.wallets.insert(
                *CURRENT_CHAIN_ID,
                PrivateKey::Encrypted(encrypted_wallet_data),
            );
            state.save();

            if let Ok(parsed_wallet) = private_key.parse::<LocalWallet>() {
                println!(
                    "Loaded and encrypted wallet with address: {:?}",
                    parsed_wallet.address()
                );
            } else {
                println!("Failed to parse wallet key, try again.");
            }
            *gamelord_caller = None;
        }
        Action::DecryptWallet(password) => {
            if let Some(PrivateKey::Encrypted(encrypted_key)) = state.wallets.get(&CURRENT_CHAIN_ID)
            {
                match decrypt_data(&encrypted_key, &password) {
                    Ok(decrypted_key) => match String::from_utf8(decrypted_key)
                        .ok()
                        .and_then(|wd| wd.parse::<LocalWallet>().ok())
                    {
                        Some(parsed_wallet) => {
                            println!(
                                "Decrypted wallet with address: {:?}",
                                parsed_wallet.address()
                            );
                            let serializable_wallet = SerializableWallet::from(parsed_wallet);
                            state.wallets.insert(
                                *CURRENT_CHAIN_ID,
                                PrivateKey::Decrypted(serializable_wallet.clone()),
                            );
                            state.save();
                            if let Some(caller) = Caller::new(
                                *CURRENT_CHAIN_ID,
                                serializable_wallet.private_key.as_str(),
                            ) {
                                *gamelord_caller = Some(GamelordCaller {
                                    caller: caller,
                                    contract_address: CONTRACT_ADDRESS.to_string(),
                                });
                            } else {
                                println!("Failed to create caller, try again.");
                                *gamelord_caller = None;
                            }
                        }
                        None => println!("Failed to parse wallet, try again."),
                    },
                    Err(_) => println!("Decryption failed, try again."),
                }
            } else {
                println!("no wallet for chainid {}", *CURRENT_CHAIN_ID);
            }
        } // _ => println!("Invalid message"),
    }
    return Ok(());
}

fn handle_worker_message(
    message: Message,
) -> anyhow::Result<()> {
    match serde_json::from_slice::<WorkerStatus>(message.body())? {
        WorkerStatus::Done => {
            clear_worker_address(&WORKER_ADDRESS);
            println!("Received status: done from worker");
            return Ok(());
        }
        _ => {
            println!("Received unknown message from worker: {:?}", message);
        }
    }
    Ok(())
}

fn handle_message(
    state: &mut State,
    gamelord_caller: &mut Option<GamelordCaller>,
    ws_channel_id: &mut Option<u32>,
    our: &Address,
) -> anyhow::Result<()> {
    let message = await_message()?;
    // Check if the source of the message is the worker address
    if let Some(worker_address) = get_worker_address(&WORKER_ADDRESS) {
        if message.source() == &worker_address {
            return handle_worker_message(message);
        }
    }
    if let "http_server:distro:sys" | "http_client:distro:sys" =
        message.source().process.to_string().as_str()
    {
        println!("HTTP request received.");
        return handle_http_request(state, gamelord_caller, ws_channel_id, &message, our);
    }
    if message.is_local(&message.source()) {
        println!("Local message received from: {:?}", message.source());

        if message.source().process.package_name == "terminal" {
            return handle_terminal_message(state, gamelord_caller, ws_channel_id, &message);
        }
        if message.source().process.package_name == "mcclient" {
            return handle_mcclient_request(state, gamelord_caller, ws_channel_id, &message, our);
        }

        handle_driver_message(state, gamelord_caller, ws_channel_id, &message, our)?;
    } else {
        println!("Message from invalid source: {:?}", message.source());
    }
    Ok(())
}

call_init!(init);
fn init(our: Address) {
    println!("{our}: gamelord started");
    let mut ws_channel_id: Option<u32> = None;
    bind_ws_path("/", true, false).unwrap();

    let _ = http::serve_ui(&our, "ui", true, false, vec!["/"]);
    // let _ = http::serve_ui(&our, "../mcstructs/ui", true, false, vec!["/"]);
    for path in [
        "/api/loadWorld",
        "/world_config",
        "/api/deleteWorld",
        "/api/editLobby",
        "/api/clearTeams",
        "/api/lockGame",
        "/api/disperseFunds",
        "/api/loadFolder",
    ] {
        http::bind_http_path(path, true, false).expect("failed to bind http path");
    }
    //http::serve_index_html(&our, "ui", true, false, vec!["/"]).unwrap();

    let _ = Request::to(("our", "eth", "distro", "sys"))
        .body(serde_json::to_vec(&EthConfigAction::AddProvider(ProviderConfig {
            chain_id: *CURRENT_CHAIN_ID,
            trusted: true,
            provider: RPC_URL.clone(),
        })).unwrap())
        .send();
    let mut state = State::fetch().unwrap_or_else(|| State::new(&our));

    let mut gamelord_caller: Option<GamelordCaller> = None;
    if let Some(PrivateKey::Decrypted(wallet)) = state.wallets.get(&CURRENT_CHAIN_ID) {
        gamelord_caller = Some(GamelordCaller {
            caller: Caller::new(*CURRENT_CHAIN_ID, &wallet.private_key).unwrap(),
            contract_address: CONTRACT_ADDRESS.to_string(),
        });
    }

    loop {
        match handle_message(&mut state, &mut gamelord_caller, &mut ws_channel_id, &our) {
            Ok(()) => {}
            Err(e) => {
                println!("error from somewhere: {:?}", e);
            }
        };
    }
}
