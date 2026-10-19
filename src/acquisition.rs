use std::fs::{File, OpenOptions};
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::{cp2110, protocol::PacketParser};

const GET_NAME: [u8; 6] = [
    0xAB, 0xCD, 0x03, 0x5F, 0x01, 0xDA,
];

const QUERY_TIMEOUT: Duration = Duration::from_secs(3);

pub enum Command {
    QueryDevice(String),
    Shutdown,
}

#[derive(Debug)]
pub enum AcquisitionEvent {
    Connecting,
    DeviceIdentified(String),
    Timeout,
    Error(String),
}

pub fn spawn_worker(
    command_rx: Receiver<Command>,
    event_tx: Sender<AcquisitionEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Ok(command) = command_rx.recv() {
            match command {
                Command::QueryDevice(path) => {
                    query_device(&path, &event_tx);
                }
                Command::Shutdown => break,
            }
        }
    })
}

fn query_device(
    path: &str,
    event_tx: &Sender<AcquisitionEvent>,
) {
    let _ = event_tx.send(AcquisitionEvent::Connecting);

    match query_device_inner(path) {
        Ok(Some(name)) => {
            let _ = event_tx.send(
                AcquisitionEvent::DeviceIdentified(name),
            );
        }

        Ok(None) => {
            let _ = event_tx.send(AcquisitionEvent::Timeout);
        }

        Err(error) => {
            let _ = event_tx.send(
                AcquisitionEvent::Error(error.to_string()),
            );
        }
    }
}

fn query_device_inner(path: &str) -> io::Result<Option<String>> {
    let mut device = open_device(path)?;

    cp2110::write_uart(&mut device, &GET_NAME)?;

    read_device_name(&mut device, QUERY_TIMEOUT)
}

fn open_device(path: &str) -> io::Result<std::fs::File> {
    let device = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;

    cp2110::configure(&device)?;

    Ok(device)
}

fn read_device_name(
    device: &mut File,
    timeout: Duration,
) -> io::Result<Option<String>> {
    let mut parser = PacketParser::new();

    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        let Some(bytes) = cp2110::read_uart(device)? else {
            continue;
        };

        for byte in bytes {
            let Some(payload) = parser.push(byte) else {
                continue;
            };

            if let Some(name) = parse_device_name(&payload) {
                return Ok(Some(name));
            }
        }
    }

    Ok(None)
}

fn parse_device_name(payload: &[u8]) -> Option<String> {
    let name = std::str::from_utf8(payload).ok()?;

    (!name.is_empty()
        && name.chars().all(|c| c.is_ascii_graphic()))
        .then(|| name.to_string())
}
