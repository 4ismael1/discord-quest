use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::process;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;

/// Connect to Discord IPC and set Rich Presence activity for the given app id.
/// Blocks the calling thread — must be run in a background thread.
pub fn connect_and_set_presence(app_id: &str) {
    loop {
        match try_connect_and_run(app_id) {
            Ok(()) => return,
            Err(_) => {
                std::thread::sleep(Duration::from_secs(5));
            }
        }
    }
}

fn try_connect_and_run(app_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut pipe = open_pipe()?;

    let handshake = format!(r#"{{"v":1,"client_id":"{}"}}"#, app_id);
    send_frame(&mut pipe, OP_HANDSHAKE, handshake.as_bytes())?;
    let _ready = read_frame(&mut pipe)?;

    let pid = process::id();
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let activity = build_activity_payload(pid, start, "init");
    send_frame(&mut pipe, OP_FRAME, activity.as_bytes())?;
    let _ack = read_frame(&mut pipe)?;

    // Keep alive: re-send the activity every 30s with the SAME start timestamp
    // so the elapsed counter in Discord keeps growing.
    let mut counter: u64 = 0;
    loop {
        std::thread::sleep(Duration::from_secs(30));
        counter += 1;
        let payload = build_activity_payload(pid, start, &format!("ka-{}", counter));
        if send_frame(&mut pipe, OP_FRAME, payload.as_bytes()).is_err() {
            return Err("Pipe write failed".into());
        }
        let _ = read_frame(&mut pipe);
    }
}

fn build_activity_payload(pid: u32, start: u64, nonce: &str) -> String {
    format!(
        r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":{{"type":0,"timestamps":{{"start":{}}}}}}},"nonce":"dq-{}"}}"#,
        pid, start, nonce
    )
}

fn open_pipe() -> Result<std::fs::File, Box<dyn std::error::Error>> {
    for i in 0..10 {
        let path = format!(r"\\.\pipe\discord-ipc-{}", i);
        if let Ok(file) = OpenOptions::new().read(true).write(true).open(&path) {
            return Ok(file);
        }
    }
    Err("No Discord IPC pipe available".into())
}

fn send_frame(
    pipe: &mut std::fs::File,
    opcode: u32,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let len = payload.len() as u32;
    let mut buf = Vec::with_capacity(8 + payload.len());
    buf.extend_from_slice(&opcode.to_le_bytes());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(payload);
    pipe.write_all(&buf)?;
    pipe.flush()?;
    Ok(())
}

fn read_frame(pipe: &mut std::fs::File) -> Result<(u32, Vec<u8>), Box<dyn std::error::Error>> {
    let mut header = [0u8; 8];
    pipe.read_exact(&mut header)?;
    let opcode = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let length = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let mut payload = vec![0u8; length];
    pipe.read_exact(&mut payload)?;
    Ok((opcode, payload))
}
