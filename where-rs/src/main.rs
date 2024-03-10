use std::cmp::max;
use std::net::UdpSocket;
use std::io::ErrorKind;
use std::time::Duration;
use where_shared::error::{WhereError, WhereResult};
use where_shared::{MAX_PAYLOAD_LENGTH, SessionCollection, WHERED_MAGIC};
use chrono::prelude::*;

pub const TIMEOUT: Duration = Duration::from_millis(2000);
pub const MAX_SEND_RETRIES: usize = 3;

fn main() {
    if let Err(e) = start_client() {
        eprintln!("where: {}", e);
        std::process::exit(1);
    }
}

fn start_client() -> WhereResult<()> {
    let servers = ["127.0.0.1:15"];
    let mut entries = vec![];

    for server in servers {
        entries.extend(process_server(server)?.into_vec());
    }

    entries.sort_by_key(|s| s.login_time);
    entries.sort_by_key(|s| !s.active);

    let max_host_length = entries.iter()
        .max_by_key(|s| match &s.host {
            Some(remote) => remote,
            None => ""
        })
        .unwrap()
        .host.clone().unwrap().len();
    let max_username_length = entries.iter()
        .max_by_key(|s| s.user.as_str())
        .unwrap()
        .user.len();
    let max_tty_length = entries.iter()
        .max_by_key(|s| s.tty.as_str())
        .unwrap()
        .tty.len();
    let max_pid_length = entries.iter()
        .max_by_key(|s| s.pid.to_string())
        .unwrap()
        .pid.to_string().len();
    let max_remote_length = entries.iter()
        .max_by_key(|s| match &s.remote {
            Some(remote) => remote,
            None => "Local"
        })
        .unwrap()
        .remote.clone().unwrap_or("Local".to_string()).len();

    let max_host_length = max(max_host_length, 4);
    let max_remote_length = max(max_remote_length, 6);
    let max_username_length = max(max_username_length, 4);
    let max_tty_length = max(max_tty_length, 3);
    let max_pid_length = max(max_pid_length, 3);

    println!("Act  Host{}  Source{}  User{}  TTY{}  PID{}  Since",
             " ".repeat(max_host_length - 4),
             " ".repeat(max_remote_length - 6),
             " ".repeat(max_username_length - 4),
             " ".repeat(max_tty_length - 3),
             " ".repeat(max_pid_length - 3),
    );

    for session in entries {
        let active_str = if session.active {
            "*"
        } else {
            " "
        };
        let host_str = session.host.unwrap_or("".to_string());
        let remote_str = session.remote.unwrap_or("Local".to_string());

        let datetime = DateTime::from_timestamp(session.login_time, 0).unwrap();
        let time_str = datetime.format("%Y-%m-%d %H:%M:%S");

        println!(" {}   {}{}  {}{}  {}{}  {}{}  {}{}  {}",
                 active_str,
                 host_str,
                 " ".repeat(max(max_host_length, host_str.len()) - host_str.len()),
                 remote_str,
                 " ".repeat(max(max_remote_length, remote_str.len()) - remote_str.len()),
                 session.user,
                 " ".repeat(max(max_username_length, session.user.len()) - session.user.len()),
                 session.tty,
                 " ".repeat(max(max_tty_length, session.tty.len()) - session.tty.len()),
                 session.pid,
                 " ".repeat(max(max_pid_length, session.pid.to_string().len()) - session.pid.to_string().len()),
                 time_str
        );
    }

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
                return Ok(SessionCollection::from_udp_payload(buf, "My Computer")?);
            },
            Err(e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(WhereError::from(e)),
        }
    }

    Err(WhereError::TimedOut(server.to_string(), MAX_SEND_RETRIES, TIMEOUT))
}
