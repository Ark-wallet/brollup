use crate::blacklist::BlacklistDirectory;
use crate::dkgops::DKGOps;
use crate::epoch_dir::dir::EpochDirectory;
use crate::into::IntoPointByteVec;
use crate::key::KeyHolder;
use crate::lp_dir::dir::LPDirectory;
use crate::nns::client::NNSClient;
use crate::noist::manager::DKGManager;
use crate::peer::PeerKind;
use crate::peer_manager::{coordinator_key, PeerManager};
use crate::registery::registery::{Registery, REGISTERY};
use crate::rollup_dir::dir::RollupDirectory;
use crate::rpc::bitcoin_rpc::validate_rpc;
use crate::rpcholder::RPCHolder;
use crate::session::ccontext::{CContextRunner, CSessionCtx};
use crate::sync::RollupSync;
use crate::tcp::tcp::{open_port, port_number};
use crate::{
    ccli, nns, tcp, Network, OperatingMode, BLIST_DIRECTORY, CSESSION_CTX, DKG_MANAGER,
    EPOCH_DIRECTORY, LP_DIRECTORY, PEER_MANAGER, ROLLUP_DIRECTORY,
};
use colored::Colorize;
use std::io::{self, BufRead};
use std::sync::Arc;

#[tokio::main]
pub async fn run(key_holder: KeyHolder, network: Network, rpc_holder: RPCHolder) {
    let mode = OperatingMode::Coordinator;

    // #1 Validate Bitcoin RPC.
    if let Err(err) = validate_rpc(&rpc_holder, network) {
        println!("{} {}", "Bitcoin RPC Error: ".red(), err);
        return;
    }

    println!("{}", "Initializing coordinator.");

    // #2 Initialize Epoch directory.
    let epoch_dir: EPOCH_DIRECTORY = match EpochDirectory::new(network) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing epoch directory.".red());
            return;
        }
    };

    // #3 Initialize LP directory.
    let lp_dir: LP_DIRECTORY = match LPDirectory::new(network) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing LP directory.".red());
            return;
        }
    };

    // #4 Initialize Registery.
    let registery: REGISTERY = match Registery::new(network) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing registery.".red());
            return;
        }
    };

    // #6 Initialize rollup directory.
    let rollup_dir: ROLLUP_DIRECTORY = match RollupDirectory::new(network) {
        Some(dir) => dir,
        None => {
            println!("{}", "Error initializing rollup directory.".red());
            return;
        }
    };

    // #7 Spawn syncer.
    {
        let network = network.clone();
        let key_holder = key_holder.clone();
        let rpc_holder = rpc_holder.clone();
        let epoch_dir = Arc::clone(&epoch_dir);
        let lp_dir = Arc::clone(&lp_dir);
        let registery = Arc::clone(&registery);
        let rollup_dir = Arc::clone(&rollup_dir);

        tokio::spawn(async move {
            let _ = rollup_dir
                .sync(
                    network,
                    &rpc_holder,
                    &key_holder,
                    &epoch_dir,
                    &lp_dir,
                    &registery,
                    None,
                )
                .await;
        });
    }

    println!("{}", "Syncing rollup.");

    // #8 Await rollup to be fully synced.
    rollup_dir.await_sync().await;

    println!("{}", "Syncing complete.");

    // #9 Check if this is the coordinator.
    if key_holder.public_key().serialize_xonly() != coordinator_key(network) {
        eprintln!("{}", "Coordinator <nsec> does not match.".red());
        return;
    }

    // #10 Initialize NNS client.
    let nns_client = NNSClient::new(&key_holder).await;

    // #11 Open port 6272 for incoming connections.
    match open_port(network).await {
        true => println!(
            "{}",
            format!("Opened port '{}'.", port_number(network)).green()
        ),
        false => (),
    }

    // #12 Run NNS server.
    {
        let nns_client = nns_client.clone();
        let _ = tokio::spawn(async move {
            let _ = nns::server::run(&nns_client, mode).await;
        });
    }

    // #13 Initialize peer manager.
    let operator_set = {
        let _epoch_dir = epoch_dir.lock().await;
        _epoch_dir.operator_set().into_xpoint_vec().expect("")
    };
    let mut peer_manager: PEER_MANAGER =
        match PeerManager::new(network, &nns_client, PeerKind::Operator, &operator_set).await {
            Some(manager) => manager,
            None => return eprintln!("{}", "Error initializing Peer manager.".red()),
        };

    // #14 Initialize DKG Manager.
    let mut dkg_manager: DKG_MANAGER = match DKGManager::new(&lp_dir) {
        Some(manager) => manager,
        None => return eprintln!("{}", "Error initializing DKG manager.".red()),
    };

    // #15 Run background preprocessing for the DKG Manager.
    dkg_manager.run_preprocessing(&mut peer_manager).await;

    // #16 Construct blacklist directory.
    let mut blacklist_dir: BLIST_DIRECTORY = match BlacklistDirectory::new(network) {
        Some(blacklist_dir) => blacklist_dir,
        None => {
            eprintln!(
                "{}",
                "Unexpected error: Failed to create blaming directory.".red()
            );
            return;
        }
    };

    // #17 Construct CSession.
    let csession_ctx: CSESSION_CTX =
        CSessionCtx::construct(&dkg_manager, &peer_manager, &blacklist_dir, &registery);

    // #18 Run CSession.
    {
        let csession_ctx = Arc::clone(&csession_ctx);
        let _ = tokio::spawn(async move {
            csession_ctx.run().await;
        });
    }

    // #19 Run TCP server.
    {
        let nns_client = nns_client.clone();
        let dkg_manager = Arc::clone(&dkg_manager);
        let csession_ctx = Arc::clone(&csession_ctx);

        let _ = tokio::spawn(async move {
            let _ = tcp::server::run(
                mode,
                network,
                &nns_client,
                &key_holder,
                &dkg_manager,
                Some(csession_ctx),
            )
            .await;
        });
    }

    // #20 Initialize CLI.
    cli(&mut peer_manager, &mut dkg_manager, &mut blacklist_dir).await;
}

pub async fn cli(
    peer_manager: &mut PEER_MANAGER,
    dkg_manager: &mut DKG_MANAGER,
    blacklist_dir: &mut BLIST_DIRECTORY,
) {
    println!(
        "{}",
        "Enter command (type help for options, type exit to quit):".cyan()
    );

    let stdin = io::stdin();
    let handle = stdin.lock();

    for line in handle.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => {
                eprintln!("{}", format!("Invalid line.").yellow());
                continue;
            }
        };

        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            // Main commands:
            "exit" => break,
            "clear" => ccli::clear::clear_command(),
            "dkg" => ccli::dkg::dkg_command(parts, peer_manager, dkg_manager).await,
            "ops" => ccli::ops::ops_command(peer_manager).await,
            "blist" => ccli::blist::blist_command(parts, blacklist_dir).await,
            _ => eprintln!("{}", format!("Unknown commmand.").yellow()),
        }
    }
}
