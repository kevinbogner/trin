use crate::cli::BridgeConfig;
use anyhow::bail;
use portalnet::socket::stun_for_external;
use std::net::SocketAddr;
use tokio::process::{Child, Command};

pub fn fluffy_handle(
    private_key: String,
    rpc_port: u16,
    udp_port: u16,
    bridge_config: BridgeConfig,
) -> anyhow::Result<Child> {
    let mut command = Command::new(bridge_config.executable_path);
    let listen_all_ips = SocketAddr::new("0.0.0.0".parse().expect("to parse ip"), udp_port);
    let ip = stun_for_external(&listen_all_ips).expect("to stun for external ip");
    command
        .kill_on_drop(true)
        .arg("--storage-size:0")
        .arg("--rpc")
        .arg(format!("--rpc-port:{rpc_port}"))
        .arg(format!("--udp-port:{udp_port}"))
        .arg(format!("--nat:extip:{}", ip.ip()))
        .arg("--network:testnet0")
        .arg("--table-ip-limit:1024")
        .arg("--bucket-ip-limit:24")
        .arg(format!("--netkey-unsafe:{private_key}"));
    if let Some(metrics_url) = bridge_config.metrics_url {
        let address = match metrics_url.host_str() {
            Some(address) => address,
            None => bail!("Invalid metrics url address"),
        };
        let port = match metrics_url.port() {
            Some(port) => port,
            None => bail!("Invalid metrics url port"),
        };
        command
            .arg("--metrics")
            .arg(format!("--metrics-address:{address}"))
            .arg(format!("--metrics-port:{port}"));
    }
    Ok(command.spawn()?)
}

pub fn trin_handle(
    private_key: String,
    rpc_port: u16,
    udp_port: u16,
    bridge_config: BridgeConfig,
) -> anyhow::Result<Child> {
    let mut command = Command::new(bridge_config.executable_path);
    command
        .kill_on_drop(true)
        .args(["--ephemeral"])
        .args(["--mb", "0"])
        .args(["--web3-transport", "http"])
        .args(["--unsafe-private-key", &private_key])
        .args([
            "--web3-http-address",
            &format!("http://127.0.0.1:{rpc_port}"),
        ])
        .args(["--discovery-port", &format!("{udp_port}")])
        .args(["--bootnodes", "default"]);
    if let Some(metrics_url) = bridge_config.metrics_url {
        let url: String = metrics_url.into();
        command.args(["--enable-metrics-with-url", &url]);
    }
    Ok(command.spawn()?)
}
