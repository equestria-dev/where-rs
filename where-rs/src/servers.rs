use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;
use whrd::error::{WhereError, WhereResult};
use whrd::{MAX_PAYLOAD_LENGTH, SessionCollection, WHERED_MAGIC};
use crate::config::{GlobalConfig, Server};

impl Server {
    fn get_address(&self, config: &GlobalConfig) -> WhereResult<SocketAddr> {
        let res: SocketAddr = match self.endpoint.to_socket_addrs() {
            Ok(mut addr) => {
                addr.find(|i| i.is_ipv4()).unwrap()
            },
            Err(_) => {
                let mut endpoint = self.endpoint.clone();
                let port = config.port.to_string();

                endpoint.push(':');
                endpoint.push_str(&port);
                endpoint.to_socket_addrs()?.find(|i| i.is_ipv4()).unwrap()
            }
        };

        Ok(res)
    }

    fn create_socket(&self, address: &SocketAddr, timeout: Duration) -> WhereResult<UdpSocket> {
        let socket = UdpSocket::bind(if address.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        })?;
        socket.set_read_timeout(Some(timeout))?;

        Ok(socket)
    }

    fn attempt_fetch(socket: &UdpSocket, address: &SocketAddr, mut buf: [u8; MAX_PAYLOAD_LENGTH], label: &str) -> WhereResult<Option<SessionCollection>> {
        socket.send_to(&WHERED_MAGIC, address)?;

        match socket.recv_from(&mut buf) {
            Ok(_) => {
                let collection = SessionCollection::from_udp_payload(buf, label)?;
                Ok(Some(collection))
            },
            Err(e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(WhereError::from(e)),
        }
    }

    pub fn process(&self, config: &GlobalConfig) -> WhereResult<SessionCollection> {
        let label = self.label.clone().unwrap_or(self.endpoint.to_owned());
        let retries = self.max_retries.unwrap_or(config.max_retries);
        let address = self.get_address(config)?;
        let timeout = Duration::from_millis(self.timeout.unwrap_or(config.timeout));
        let socket = self.create_socket(&address, timeout)?;
        let buf = [0; MAX_PAYLOAD_LENGTH];

        for _ in 0..retries {
            match Self::attempt_fetch(&socket, &address, buf, &label)? {
                Some(c) => return Ok(c),
                None => continue
            };
        }

        Err(WhereError::TimedOut(self.endpoint.to_string(), address.to_string(), retries, timeout))
    }
}
