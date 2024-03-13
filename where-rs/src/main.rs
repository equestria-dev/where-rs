use std::fs;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::io::ErrorKind;
use std::time::Duration;
use where_shared::error::{WhereError, WhereResult};
use where_shared::{Session, SessionCollection, MAX_PAYLOAD_LENGTH, WHERED_MAGIC};
use chrono::prelude::*;
use serde::Deserialize;

pub const TIMEOUT: u64 = 2000;
pub const MAX_SEND_RETRIES: usize = 3;

#[derive(Deserialize, Debug)]
struct Config {
    global: Option<GlobalConfig>,
    server: Vec<ServerConfig>
}

#[derive(Deserialize, Debug)]
struct ServerConfig {
    endpoint: String,
    label: Option<String>,
    timeout: Option<u64>,
    max_retries: Option<usize>,
    failsafe: Option<bool>
}

#[derive(Deserialize, Debug, Clone, Default)]
struct GlobalConfig {
    timeout: Option<u64>,
    max_retries: Option<usize>,
    include_inactive: Option<bool>,
    port: Option<u16>,
    source: Option<String>
}

fn main() {
    if let Err(e) = start_client() {
        eprintln!("where: {}", e);
        std::process::exit(1);
    }
}

fn start_client() -> WhereResult<()> {
    // TODO: Make it load from an actual path: /etc/where.toml, or ~/.where.toml if it exists
    let config_path = "./config.toml";

    let config: Config = toml::from_str(&fs::read_to_string(config_path).unwrap_or_else(|e| {
        eprintln!("where: Failed to open configuration file: {e}");
        std::process::exit(1);
    })).unwrap_or_else(|e| {
        eprintln!("where: Failed to parse configuration file: {e}");
        std::process::exit(1);
    });

    println!("{:?}", config);
    let global_config = config.global.unwrap_or_default();

    let servers: Vec<ServerConfig> = config.server;
    let mut sessions = vec![];

    for server in servers {
        // I know using .clone() sucks!
        let res = match process_server(&server, global_config.clone()) {
            Ok(data) => {
                data
            }
            Err(e) => {
                eprintln!("where: {e}");

                if !server.failsafe.unwrap_or(false) {
                    std::process::exit(1);
                }

                SessionCollection::get_empty()
            }
        };

        sessions.extend(res.into_vec());
    }

    print_summary(sessions, global_config);
    Ok(())
}

fn process_server(server: &ServerConfig, config: GlobalConfig) -> WhereResult<SessionCollection> {
    let label = server.label.clone().unwrap_or(server.endpoint.to_owned());
    let timeout = Duration::from_millis(server.timeout.unwrap_or(config.timeout.unwrap_or(TIMEOUT)));
    let retries = server.max_retries.unwrap_or(config.max_retries.unwrap_or(MAX_SEND_RETRIES));

    let address: SocketAddr = match server.endpoint.to_socket_addrs() {
        Ok(addr) => addr.as_slice()[0],
        Err(_) => {
            let mut endpoint = server.endpoint.clone();
            endpoint.push_str(&format!(":{}", config.port.unwrap_or(15)));
            endpoint.to_socket_addrs()?.as_slice()[0]
        }
    };

    let socket = UdpSocket::bind(if address.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    })?;
    socket.set_read_timeout(Some(timeout))?;

    let mut buf = [0; MAX_PAYLOAD_LENGTH];

    for _ in 0..retries {
        socket.send_to(&WHERED_MAGIC, address)?;

        match socket.recv_from(&mut buf) {
            Ok(_) => {
                let collection = SessionCollection::from_udp_payload(buf, &label)?;
                return Ok(collection);
            },
            Err(e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(WhereError::from(e)),
        }
    }

    Err(WhereError::TimedOut(server.endpoint.to_string(), address.to_string(), retries, timeout))
}

fn print_summary(mut sessions: Vec<Session>, config: GlobalConfig) {
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
    sessions.sort_by_key(|s| !s.active); // We want active first

    const ACTIVE_PADDING: usize = 2;
    let host_padding = max_key_with_min(&sessions, |s| s.host.as_deref().map_or(0, |str| str.len()), 5);
    let remote_padding = max_key_with_min(&sessions, |s| s.remote.as_deref().map_or(0, |str| str.len()), 7);
    let username_padding = max_key_with_min(&sessions, |s| s.user.len(), 5);
    let tty_padding = max_key_with_min(&sessions, |s| s.tty.len(), 4);
    let pid_padding = max_key_with_min(&sessions, |s| s.pid.abs().checked_ilog10().unwrap_or_default() + 1 + (s.pid < 0) as u32, 4);

    println!("{:pad_0$} {:<pad_1$} {:<pad_2$} {:<pad_3$} {:<pad_4$} {:<pad_5$} Since",
              "Act",
              "Host",
              "Source",
              "User",
              "TTY",
              "PID",
              pad_0 = ACTIVE_PADDING,
              pad_1 = host_padding,
              pad_2 = remote_padding,
              pad_3 = username_padding,
              pad_4 = tty_padding,
              pad_5 = pid_padding as usize);

    for session in sessions {
        if !config.include_inactive.unwrap_or(true) && !session.active {
            continue;
        }

        let active = if session.active {
            '*'
        } else {
            ' '
        };

        let host = session.host.unwrap_or_else(|| ' '.to_string());
        let remote = session.remote.unwrap_or_else(|| config.source.clone().unwrap_or("Local".to_string()));

        let datetime = DateTime::from_timestamp(session.login_time, 0).unwrap();
        let time = datetime.format("%Y-%m-%d %H:%M:%S");

        println!(" {:<pad_0$} {:<pad_1$} {:<pad_2$} {:<pad_3$} {:<pad_4$} {:<pad_5$} {}",
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
