use std::net::UdpSocket;
use std::io::ErrorKind;
use std::time::Duration;
use where_shared::error::{WhereError, WhereResult};
use where_shared::{MAX_PAYLOAD_LENGTH, SessionCollection, WHERED_MAGIC};

pub const TIMEOUT: Duration = Duration::from_millis(2000);
pub const MAX_SEND_RETRIES: usize = 3;

fn main() {
    if let Err(e) = start_client() {
        eprintln!("where: {}", e);
        std::process::exit(1);
    }
}

fn start_client() -> WhereResult<()> {
    println!("{:?}", process_server("127.0.0.1:15")?);
    Ok(())
}

fn process_server(server: &str) -> WhereResult<SessionCollection> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(TIMEOUT))?;

    let mut buf = [0; MAX_PAYLOAD_LENGTH];

    for _ in 0..MAX_SEND_RETRIES {
        socket.send_to(&WHERED_MAGIC, server)?;

        match socket.recv_from(&mut buf) {
            Ok(_) => {
                return Ok(SessionCollection::from_udp_payload(buf)?);
            },
            Err(e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(WhereError::from(e)),
        }
    }

    Err(WhereError::TimedOut(server.to_string(), MAX_SEND_RETRIES, TIMEOUT))
}
