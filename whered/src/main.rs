use std::net::UdpSocket;
use where_shared::error::WhereResult;
use where_shared::{SessionCollection, WHERED_MAGIC};

fn main() {
    if let Err(e) = run_server() {
        eprintln!("whered: {}", e);
        std::process::exit(1);
    }
}

fn run_server() -> WhereResult<()> {
    let socket = UdpSocket::bind("0.0.0.0:15")?;
    println!("Now listening on 0.0.0.0:15");

    loop {
        if let Err(e) = handle_request(&socket) {
            eprintln!("whered: {}", e);
        }
    }
}

fn handle_request(socket: &UdpSocket) -> WhereResult<()> {
    let mut buf = [0; WHERED_MAGIC.len()];

    let (_, src) = socket.recv_from(&mut buf)?;
    println!("{src}: New client!");

    let sessions = SessionCollection::fetch();
    let buf = sessions.to_udp_payload()?;

    socket.send_to(&buf, src)?;
    println!("{src}: Completed request within {} bytes", buf.len());

    Ok(())
}