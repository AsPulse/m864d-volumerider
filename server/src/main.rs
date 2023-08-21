use mixer_server::MixerServer;

mod mixer_server;
mod log;

#[tokio::main]
async fn main() {
    let mixer_server = MixerServer {
        host_communicate: "192.168.14.1:3000".to_string(),
        host_levelmeter: "192.168.14.1:3001".to_string(),
    };

    mixer_server.connect().await;
}
