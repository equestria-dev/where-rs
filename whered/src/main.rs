use std::net::UdpSocket;
use where_shared::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:15").expect("Could not bind to port 15, is another instance of whered running?");

    loop {
        let mut buf = [0; WHERED_MAGIC.len()];
        let Ok((_, src)) = socket.recv_from(&mut buf) else {
            eprintln!("Failed to receive data from the client, ignoring");
            continue
        };

        println!("{src}: New client!");

        let sessions = SessionCollection::fetch();
        if let Err(_) = socket.send_to(&*sessions.into_bytes(), src) {
            eprintln!("{src}: Failed to send data back to the client, ignoring");
        } else {
            println!("{src}: Completed request");
        }
    }
}