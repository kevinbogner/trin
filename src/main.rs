#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate tracing;

mod cli;
use cli::TrinConfig;
mod jsonrpc;
use jsonrpc::launch_trin;
use log::info;
use std::env;
use std::time::Duration;

mod portalnet;
use portalnet::protocol::{PortalnetConfig, PortalnetProtocol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let trin_config = TrinConfig::new();

    let infura_project_id = match env::var("TRIN_INFURA_PROJECT_ID") {
        Ok(val) => val,
        Err(_) => panic!(
            "Must supply Infura key as environment variable, like:\n\
            TRIN_INFURA_PROJECT_ID=\"your-key-here\" trin"
        ),
    };

    let listen_port = trin_config.discovery_port;
    let bootnode_enrs = trin_config
        .bootnodes
        .iter()
        .map(|nodestr| nodestr.parse().unwrap())
        .collect();

    let portalnet_config = PortalnetConfig {
        listen_port,
        bootnode_enrs,
        ..Default::default()
    };

    let web3_server_task = tokio::task::spawn_blocking(|| {
        launch_trin(trin_config, infura_project_id);
    });

    info!(
        "About to spawn portal p2p with boot nodes: {:?}",
        portalnet_config.bootnode_enrs
    );
    tokio::spawn(async move {
        let mut p2p = PortalnetProtocol::new(portalnet_config).await.unwrap();
        // hacky test: make sure we establish a session with the boot node
        p2p.ping_bootnodes().await.unwrap();

        // TODO Probably some new API like p2p.maintain_network() that blocks forever
        tokio::time::sleep(Duration::from_secs(86400 * 365 * 10)).await;
    })
    .await
    .unwrap();

    web3_server_task.await.unwrap();
    Ok(())
}
