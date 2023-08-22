use std::time::Duration;

use chrono::Utc;
use mixer_server::{MixerServer, MixerChannel, MixerCommand, Level};

use crate::log::log_time_role;

mod mixer_server;
mod log;

macro_rules! round {
    ($x:expr, $scale:expr) => (($x * $scale).round() / $scale)
}

static TARGET: f64 = -15.0;
static START: f64 = -25.0;
static INC_FACTOR: f64 = 0.5;
static DEC_FACTOR: f64 = 0.75;

#[tokio::main]
async fn main() {
    let mixer_server = MixerServer {
        host_communicate: "192.168.14.1:3000".to_string(),
        host_levelmeter: "192.168.14.1:3001".to_string(),
    };


    let target_channel = MixerChannel::MonoIn(6);
    let mut fader_target: f64;
    let mut fader_value = 0.0;
    let mut previous_level = Level { time: Utc::now(), channel: target_channel.clone(), level: -35.0 };


    let (mut server, mut joinset) = mixer_server.connect().await;
    loop {
        tokio::time::sleep(Duration::from_millis(200)).await;
        if let Err(e) = server.command.send( MixerCommand::SendLevel {
            channel: target_channel.clone(),
        }).await {
            println!("Error: {:?}", e);
        }

        let velocity;
        (fader_target, velocity) = if let Some(level) = server.level.recv().await {
            if level.channel == target_channel {
                let result = calc_target(&previous_level, &level);
                previous_level = level;
                result
            } else {
                (0.0, -1.0)
            }
        } else {
            (0.0, -1.0)
        };

        if (fader_value - fader_target).abs() < 1.0 {
            fader_value = fader_target;
        } else {
            if fader_value > fader_target {
                fader_value += (fader_target - fader_value) * DEC_FACTOR;
            } else {
                fader_value += (fader_target - fader_value) * INC_FACTOR;
            }
        }

        println!("{} {:?}dB -> {:?}dB (vel {:?})", log_time_role("FADER"), round!(fader_value, 10.0), round!(fader_target, 10.0), round!(velocity, 0.0));
        if let Err(e) = server.command.send(MixerCommand::ChangeLevel { channel: target_channel.clone(), gain: fader_value.max(7.0) }).await {
            println!("Error: {:?}", e);
        }
        
    }


    while let Some(_) = joinset.join_next().await {}
}

fn calc_target(previous_level: &Level, level: &Level) -> (f64, f64) {
    let velocity = (level.level - previous_level.level) / ((level.time - previous_level.time).num_milliseconds() as f64) * 250.0;
    if level.level < START { return (0.0, velocity); }
    if velocity < -10.0 { return (0.0, velocity); }
    ((TARGET - (level.level + previous_level.level) / 2.0).clamp(-12.0, 15.0), velocity)
}