use tokio::{net::TcpStream, sync::{oneshot, mpsc}, io::{AsyncReadExt, AsyncWriteExt}};

use crate::log::log_time_role;

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
    pub async fn connect(&self) {
        let communicate = self.host_communicate.clone();
        let levelmeter = self.host_levelmeter.clone();
        
        //let (commu_tx, mut commu_rx) = mpsc::channel(32);
        let commu = tokio::spawn(async move {
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
                        let payload = &buf[0..len];
                        match payload {
                            [223, 1, 1] => {
                                println!("{} Recv Established {:?}", log_time_role("COMMU"), payload);
                            }
                            [255] => {
                                println!("{} Recv Keepalive {:?}", log_time_role("COMMU"), payload);
                                println!("{} Send Keepalive {:?}", log_time_role("COMMU"), [0xFF]);
                                stream.write_all(&[0xFF]).await.expect("Write failed!");
                            }
                            _ => {
                                println!("{} Recv {:?} (Unknown)", log_time_role("COMMU"), payload);
                            }
                        }
                    }
                }
            }
        });

        let level = tokio::spawn(async move {
            println!("<LEVEL> Host Connecting...");
            let addr: &str = levelmeter.as_str();
            let mut stream = TcpStream::connect(addr)
                .await
                .expect(format!("Cannot connect to LEVELMETER_HOST({0})", addr).as_str());

            let mut buf: [u8; 512] = [0; 512];
            println!("<LEVEL> Host Connected!");
            loop {
                tokio::select! {
                    Ok(len) = stream.read(&mut buf) => {
                        let payload = &buf[0..len];
                        match payload {
                            _ => {
                                println!("{} Recv {:?} (Unknown)", log_time_role("LEVEL"), payload);
                            }
                        }
                    }
                }
            }
        });

        tokio::join!(commu, level);
    }
}