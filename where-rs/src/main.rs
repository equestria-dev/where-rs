use std::net::UdpSocket;
use coreutils_core::ByteSlice;
use where_shared::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not start a UDP socket.");
    socket.send_to(&WHERED_MAGIC, "127.0.0.1:15").expect("Could not send data to the server.");

    let mut buf = [0; 1024];
    socket.recv_from(&mut buf).expect("No data to receive from the server.");

    /*let list = SessionCollection::from_bytes(buf.to_vec());
    println!("{:?}", list);*/
    println!("{}", String::from_utf8_lossy(&buf));
}