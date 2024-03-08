use coreutils_core::ByteSlice;
use coreutils_core::os::utmpx::UtmpxKind;

pub const WHERED_MAGIC: [u8; 4] = *b"WHRD";

#[derive(Debug)]
pub struct Session {
    user: String,
    pid: i32,
    tty: String,
    remote: Option<String>,
    active: bool,
    login: i64
}

#[derive(Debug)]
pub struct SessionCollection<Session> {
    inner: Vec<Session>
}

impl SessionCollection<Session> {
    pub fn fetch() -> SessionCollection<Session> {
        let mut output: SessionCollection<Session> = SessionCollection {
            inner: vec![]
        };
        let utmp = coreutils_core::os::utmpx::UtmpxSet::system();

        for item in utmp {
            if item.entry_type() != UtmpxKind::UserProcess && item.entry_type() != UtmpxKind::DeadProcess {
                continue;
            }

            let host = item.host().to_string();

            output.inner.push(Session {
                user: item.user().to_string(),
                pid: item.process_id(),
                tty: item.device_name().to_string(),
                remote: if &host == "" {
                    None
                } else {
                    Some(host)
                },
                active: item.entry_type() == UtmpxKind::UserProcess,
                login: item.timeval().tv_sec
            });
        }

        output
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        for item in self.inner {
            bytes.append(&mut item.into_bytes().to_vec());
            bytes.append(&mut vec![0, 0, 0, 0]);
        }

        bytes
    }

    /*pub fn from_bytes(bytes: Vec<u8>) -> SessionCollection<Session> {
        let entries = bytes.split([0, 0]);
        let mut final_entries: Vec<Session> = vec![];

        for item in entries {
            let parts: Vec<&[u8]> = item.split(0).collect();
            if item.split(0).count() == 0 {
                continue;
            }

            /*final_entries.push(Session {
                //pid: i32::from_be_bytes(parts[0].as_bytes()),
                //login: i64::from_be_bytes(parts[1].as_bytes()),
                pid: 0,
                login: 0,
                user: parts[2].to_string(),
                tty: parts[3].to_string(),
                remote: if parts[4] == b"\xff".to_str().unwrap() {
                    None
                } else {
                    Some(parts[4].to_string())
                },
                active: parts[5] == b"\xff".to_str().unwrap()
            });*/
        }

        SessionCollection {
            inner: final_entries
        }
    }*/
}

impl Session {
    pub fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        let mut host_bytes = self.remote.clone().unwrap_or(String::from("!")).as_bytes().to_vec();
        let mut full = vec![255];

        bytes.append(&mut self.pid.to_be_bytes().to_vec());
        bytes.append(&mut self.login.to_be_bytes().to_vec());
        bytes.append(&mut (self.user.len() as u32).to_be_bytes().to_vec());
        bytes.append(&mut self.user.as_bytes().to_vec());
        bytes.append(&mut (self.tty.len() as u32).to_be_bytes().to_vec());
        bytes.append(&mut self.tty.as_bytes().to_vec());
        bytes.append(&mut (host_bytes.len() as u32).to_be_bytes().to_vec());
        bytes.append(if let Some(_) = self.remote {
            &mut host_bytes
        } else {
            &mut full
        });
        bytes.push(if self.active {
            1
        } else {
            0
        });

        bytes
    }
}