use std::time::Duration;

use mixer_server::{MixerServer, MixerChannel, MixerCommand};

mod mixer_server;
mod log;

#[tokio::main]
async fn main() {
    let mixer_server = MixerServer {
        host_communicate: "192.168.14.1:3000".to_string(),
        host_levelmeter: "192.168.14.1:3001".to_string(),
    };

    let (server, mut joinset) = mixer_server.connect().await;
    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        if let Err(e) = server.command.send( MixerCommand::SendLevel {
            channel: MixerChannel::MonoIn(6)
        }).await {
            println!("Error: {:?}", e);
        }
    }
    while let Some(_) = joinset.join_next().await {}
}
