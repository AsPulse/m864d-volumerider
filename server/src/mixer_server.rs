use tokio::{net::TcpStream, sync::{oneshot, mpsc}, io::AsyncReadExt};

pub struct MixerServer {
    pub host_communicate: String,
    pub host_levelmeter: String,
    //connection: 
}

type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
enum MixerCommand {
    GetLevel {
        channel: MixerChannel,
        response: Responder<f32>,
    }
}

#[derive(Debug)]
enum MixerChannel {
    MonoIn(u8),
    StereoIn(u8),
}

impl MixerServer {
    pub async fn connect(&self) -> (Result<(), tokio::task::JoinError>,){
        let communicate = self.host_communicate.clone();
        //let (host_tx, mut host_rx) = mpsc::channel(32);
        let host = tokio::spawn(async move {
            println!("<COMMU> Host Connecting...");
            let addr = communicate.as_str();
            let mut stream = TcpStream::connect(addr)
                .await
                .expect(format!("Cannot connect to COMMUNICATE_HOST({0})", addr).as_str());

            let mut buf: [u8; 512] = [0; 512];
            println!("<COMMU> Host Connected!");
            loop {
                tokio::select! {
                    Ok(len) = stream.read(&mut buf) => {
                        match &buf[0..len] {
                            [223, 1, 1] => {
                                println!("<COMMU> Recv Established {:?}", &buf[0..len]);
                            }
                            [255] => {
                                println!("<COMMU> Recv Keepalive {:?}", &buf[0..len]);
                            }
                            _ => {
                                println!("<COMMU> Recv {:?} (Unknown)", &buf[0..len]);
                            }
                        }
                    }
                }
            }
        });

        return tokio::join!(host);
    }
}