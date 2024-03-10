use std::net::UdpSocket;
use std::io::ErrorKind;
use std::time::Duration;
use where_shared::error::{WhereError, WhereResult};
use where_shared::{Session, SessionCollection, MAX_PAYLOAD_LENGTH, WHERED_MAGIC};
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
    let mut sessions = vec![];

    for server in servers {
        sessions.extend(process_server(server, "My Computer")?.into_vec());
    }

    print_summary(sessions);
    Ok(())
}

fn process_server(server: &str, host: &str) -> WhereResult<SessionCollection> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(TIMEOUT))?;

    let mut buf = [0; MAX_PAYLOAD_LENGTH];

    for _ in 0..MAX_SEND_RETRIES {
        socket.send_to(&WHERED_MAGIC, server)?;

        match socket.recv_from(&mut buf) {
            Ok(_) => {
                let collection = SessionCollection::from_udp_payload(buf, host)?;
                return Ok(collection);
            },
            Err(e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(WhereError::from(e)),
        }
    }

    Err(WhereError::TimedOut(server.to_string(), MAX_SEND_RETRIES, TIMEOUT))
}

fn print_summary(mut sessions: Vec<Session>) {
    fn max_key_with_min<T, F>(sessions: &[Session], get_key: F, floor: T) -> T
    where
        T: Ord + Default,
        F: Fn(&Session) -> T
    {
        sessions.iter()
            .max_by_key(|s| get_key(s))
            .map(get_key)
            .unwrap_or_default()
            .max(floor)
    }


    sessions.sort_unstable_by_key(|s| s.login_time);
    sessions.sort_by_key(|s| s.active);

    const ACTIVE_PADDING: usize = 4;
    let host_padding = max_key_with_min(&sessions, |s| s.host.as_deref().map_or(0, |str| str.len()), 5);
    let remote_padding = max_key_with_min(&sessions, |s| s.remote.as_deref().map_or(0, |str| str.len()), 7);
    let username_padding = max_key_with_min(&sessions, |s| s.user.len(), 5);
    let tty_padding = max_key_with_min(&sessions, |s| s.tty.len(), 4);
    let pid_padding = max_key_with_min(&sessions, |s| s.pid.abs().checked_ilog10().unwrap_or_default() + 1 + (s.pid < 0) as u32, 4);

    println!("{:pad_0$} {:<pad_1$} {:<pad_2$} {:<pad_3$} {:<pad_4$} {:<pad_5$} {}",
              "Act",
              "Host",
              "Source",
              "User",
              "TTY",
              "PID",
              "Since",
              pad_0 = ACTIVE_PADDING,
              pad_1 = host_padding,
              pad_2 = remote_padding,
              pad_3 = username_padding,
              pad_4 = tty_padding,
              pad_5 = pid_padding as usize);

    for session in sessions {
        let active = if session.active {
            '*'
        } else {
            ' '
        };

        let host = session.host.unwrap_or_else(|| ' '.to_string());
        let remote = session.remote.unwrap_or_else(|| "Local".to_owned());

        let datetime = DateTime::from_timestamp(session.login_time, 0).unwrap();
        let time = datetime.format("%Y-%m-%d %H:%M:%S");

        println!("{:<pad_0$} {:<pad_1$} {:<pad_2$} {:<pad_3$} {:<pad_4$} {:<pad_5$} {}",
                active,
                host,
                remote,
                session.tty,
                session.user,
                session.pid,
                time,
                pad_0 = ACTIVE_PADDING,
                pad_1 = host_padding,
                pad_2 = remote_padding,
                pad_3 = username_padding,
                pad_4 = tty_padding,
                pad_5 = pid_padding as usize);
    }
}
