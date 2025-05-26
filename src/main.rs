use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};

use iroh::NodeId;
use ps_base64::base64;
use scatter_net::{NetConfig, NetState, ScatterNet};
use scatter_send::File;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use tokio::time::sleep;

/// A program to send or receive files and directories using `ScatterNet`.
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about = "Send or receive files and directories using ScatterNet.",
    long_about = None
)]
struct CliArgs {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Send a file or directory
    Send(SendCommand),

    /// Receive a file or directory using a ticket
    Receive(ReceiveCommand),
}

#[derive(Parser, Debug)]
struct SendCommand {
    /// The path to the file or directory to send
    #[clap(value_parser)]
    path: PathBuf,
}

#[derive(Parser, Debug)]
struct ReceiveCommand {
    /// The path to save the received file or directory
    #[clap(value_parser)]
    path: PathBuf,

    /// The ticket used to identify and authenticate the transfer
    #[clap(value_parser)]
    ticket: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = CliArgs::parse();

    // Configure network
    let mut config = NetConfig::from_file("scatter-send.toml")?;

    config.populate()?;

    let net = ScatterNet::init(config, NetState::default()).await?;

    match args.command {
        Command::Send(cmd) => {
            println!("Preparing to send: {}", cmd.path.display());

            let file = File::scatter(cmd.path, net.clone()).await?;

            let ticket = Ticket {
                file,
                node_id: net.get_node_id(),
            };

            let ticket = base64::encode(serde_json::ser::to_string(&ticket)?.as_bytes());

            println!("Ticket: {ticket}");

            eprintln!("Press CTRL+C to exit!");

            tokio::signal::ctrl_c().await?;
        }
        Command::Receive(cmd) => {
            println!(
                "Preparing to receive file to: {} using ticket: {}",
                cmd.path.display(),
                cmd.ticket
            );
            // Implement the receive logic
            let ticket: Ticket = from_slice(&base64::decode(cmd.ticket.as_bytes()))?;

            let peer = net.connect_to(ticket.node_id, None).await?;

            sleep(Duration::from_secs(1)).await;

            // validation connection
            eprintln!("Validating connection...");
            let rtt = peer.ping(Duration::from_secs(1)).await?;

            eprintln!("Connection validated, rtt={rtt}");
            ticket.file.collect(net, &cmd.path).await?;

            println!("File received and saved to: {}", cmd.path.display());
            println!("Size: {} bytes", ticket.file.size);
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ScatterSendError {
    #[error("Invalid path")]
    InvalidPath,
}

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub file: File,
    pub node_id: NodeId,
}
