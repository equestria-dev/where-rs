mod args;

use args::Args;
use std::net::{SocketAddr, UdpSocket};
use std::process;
use std::str::FromStr;
use clap::Parser;
use whrd::error::{WhereError, WhereResult};
use whrd::{SessionCollection, WHERED_MAGIC};

fn main() {
    let args = Args::parse();
    let listen_addr = args.listen_addr.unwrap_or(String::from("0.0.0.0:15"));

    if let Err(e) = run_server(&listen_addr) {
        eprintln!("whered: {}", e);
        process::exit(1);
    }
}

fn run_server(listen_addr: &str) -> WhereResult<()> {
    let socket_addr_result = SocketAddr::from_str(listen_addr);

    match socket_addr_result {
        Ok(socket_addr) => {
            let socket = UdpSocket::bind(socket_addr)?;
            println!("Now listening on {} port {}/udp", socket_addr.ip(), socket_addr.port());

            loop {
                if let Err(e) = handle_request(&socket) {
                    eprintln!("whered: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("whered: {}", WhereError::from(e));
            process::exit(1);
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
