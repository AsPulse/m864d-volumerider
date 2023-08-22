use std::time::Duration;

use chrono::Utc;
use tokio::{net::TcpStream, sync::mpsc, io::{AsyncReadExt, AsyncWriteExt}, task::JoinSet};

use crate::log::log_time_role;

pub struct MixerServer {
    pub host_communicate: String,
    pub host_levelmeter: String,
}

pub struct MixerConnection {
    pub command: mpsc::Sender<MixerCommand>,
    pub level: mpsc::Receiver<Level>,
}

pub struct Level {
    pub time: chrono::DateTime<Utc>,
    pub channel: MixerChannel,
    pub level: f64,
}


#[derive(Debug)]
pub enum MixerCommand {
    SendLevel {
        channel: MixerChannel,
    },
    ChangeLevel {
        channel: MixerChannel,
        gain: f64,
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MixerChannel {
    MonoIn(u8),
    StereoIn(u8),
}

impl MixerChannel {
    pub fn to_string(&self) -> String {
        match self {
            Self::MonoIn(ch) => format!("M_In#{}", ch),
            Self::StereoIn(ch) => format!("S_In#{}", ch),
        }
    }
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            Self::MonoIn(ch) => [0x00, *ch],
            Self::StereoIn(ch) => [0x01, *ch],
        }
    }
    pub fn from_bytes(bytes: [&u8; 2]) -> Self {
        match bytes {
            [0x00, ch] => Self::MonoIn(*ch),
            [0x01, ch] => Self::StereoIn(*ch),
            _ => unimplemented!("{:?} represents no kind of channel.", bytes)
        }
    }
}

impl MixerServer {
    pub async fn connect(&self) -> (MixerConnection, JoinSet<()>) {
        let mut join_set = JoinSet::new();
        
        let (cmd_tx, mut cmd_rx) = mpsc::channel(32);
        let (level_tx, level_rx) = mpsc::channel(32);

        let mixer_connection = MixerConnection {
            level: level_rx,
            command: cmd_tx,
        };

        println!("<COMMU> Host Connecting...");
        let commu_stream = TcpStream::connect(&self.host_communicate)
            .await
            .expect(format!("Cannot connect to COMMUNICATE_HOST({0})", &self.host_communicate).as_str());

        join_set.spawn(async move {
            let mut stream = commu_stream;
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
                                stream.write_all(&[0xFF]).await.unwrap();
                            }
                            _ => {
                                println!("{} Recv {:?} (Unknown)", log_time_role("COMMU"), payload);
                            }
                        }
                    },
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd {
                            MixerCommand::SendLevel { channel } => {
                                let [attr, num] = channel.to_bytes();
                                let payload = [0xf0, 0x03, 0x17, attr, num];
                                println!("{} Send Request {} Level {:?}", log_time_role("COMMU"), channel.to_string(), payload);
                                stream.write_all(&payload).await.unwrap();
                            }
                            MixerCommand::ChangeLevel { channel, gain } => {
                                let [attr, num] = channel.to_bytes();
                                let lev = (gain.clamp(-38.0, 10.0) + 53.0).round() as u8;
                                let payload = [0x91, 0x03, attr, num, lev];
                                println!("{} Send Set {} Fader {:?}dB {:?}", log_time_role("COMMU"), channel.to_string(), gain, payload);
                                stream.write_all(&payload).await.unwrap();
                            }
                        }
                    }
                }
            }
        });

        println!("<LEVEL> Host Connecting...");
        let level_stream = TcpStream::connect(&self.host_levelmeter)
            .await
            .expect(format!("Cannot connect to LEVELMETER_HOST({0})", &self.host_levelmeter).as_str());

        join_set.spawn(async move {
            let mut stream = level_stream;
            let mut buf: [u8; 512] = [0; 512];
            println!("<LEVEL> Host Connected!");
            loop {
                tokio::select! {
                    Ok(len) = stream.read(&mut buf) => {
                        let payload = &buf[0..len];
                        match payload {
                            [0xe6, 0x04, 0x00, attr, num, meter] => {
                                let channel = MixerChannel::from_bytes([attr, num]);
                                let dbu: f64 = (*meter as f64) - 48.0;
                                println!("{} Recv {} Level is {:?}dBu {:?}", log_time_role("LEVEL"), channel.to_string(), dbu, payload);
                                level_tx.send(Level {
                                    time: Utc::now(),
                                    channel,
                                    level: dbu,
                                }).await.unwrap();
                            },
                            _ => {
                                println!("{} Recv {:?} (Unknown)", log_time_role("LEVEL"), payload);
                            }
                        }
                    },
                    _ = tokio::time::sleep(Duration::from_millis(5000)) => {
                        println!("{} Send Keepalive {:?}", log_time_role("LEVEL"), [0xFF]);
                        stream.write_all(&[0xFF]).await.unwrap();
                    }
                }
            }
        });

        (mixer_connection, join_set)
    }
}