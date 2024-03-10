use std::io::Cursor;
use coreutils_core::os::utmpx::*;
use std::io::Read;
use crate::error::{EncodeDecodeError, EncodeDecodeResult};

pub mod error;

pub const WHERED_MAGIC: [u8; 4] = *b"WHRD";
pub const MAX_USER_TTY_LENGTH: usize = 32;
pub const MAX_REMOTE_LENGTH: usize = 64;
pub const MAX_ENTRY_LENGTH: usize = MAX_REMOTE_LENGTH + MAX_USER_TTY_LENGTH * 2 + 25;
pub const MAX_PAYLOAD_LENGTH: usize = 65501;
pub const MAX_PAYLOAD_ENTRIES: usize = MAX_PAYLOAD_LENGTH / MAX_ENTRY_LENGTH;

#[derive(Debug)]
pub struct Session {
    pub host: Option<String>,
    pub user: String,
    pub pid: i32,
    pub tty: String,
    pub remote: Option<String>,
    pub active: bool,
    pub login_time: i64
}

#[derive(Debug)]
pub struct SessionCollection {
    inner: Vec<Session>
}

impl SessionCollection {
    pub fn fetch() -> Self {
        let inner: Vec<Session> = UtmpxSet::system()
            .into_iter()
            .filter(|utmpx| utmpx.entry_type() == UtmpxKind::UserProcess || utmpx.entry_type() == UtmpxKind::DeadProcess)
            .map(Session::from)
            .collect();

        Self {
            inner
        }
    }

    pub fn into_vec(self) -> Vec<Session> {
        self.inner
    }

    pub fn to_udp_payload(self) -> EncodeDecodeResult<Vec<u8>> {
        println!("Encoding payload with {} entries", self.inner.len());

        let mut bytes: Vec<u8> = vec![];
        bytes.extend(&WHERED_MAGIC);

        let entry_count = (self.inner.len() as u16).to_be_bytes();
        bytes.extend(&entry_count);

        for item in self.inner {
            let entry = item.to_udp_payload();

            if entry.len() > MAX_ENTRY_LENGTH {
                return Err(EncodeDecodeError::InvalidEntryLength(entry.len()));
            }

            bytes.extend(entry);
        }

        if bytes.len() > MAX_PAYLOAD_LENGTH {
            Err(EncodeDecodeError::InvalidPayloadLength(bytes.len()))
        } else {
            Ok(bytes)
        }
    }

    pub fn from_udp_payload(buffer: [u8; MAX_PAYLOAD_LENGTH], host: &str) -> EncodeDecodeResult<Self> {
        let mut buf = Cursor::new(buffer);
        let mut inner = vec![];
        let mut magic = [0u8; 4];
        let mut length = [0u8; 2];

        Session::read_field(&mut buf, &mut magic)?;
        Session::read_field(&mut buf, &mut length)?;
        let entry_count = u16::from_be_bytes(length);

        if magic != WHERED_MAGIC {
            return Err(EncodeDecodeError::BadMagic(magic));
        }

        for _ in 0..entry_count {
            inner.push(Session::from_udp_payload(&mut buf, &host)?);
        }

        if inner.len() != entry_count as usize {
            return Err(EncodeDecodeError::IncorrectEntryCount);
        }

        Ok(Self {
            inner
        })
    }
}

impl Session {
    pub fn from_udp_payload(cursor: &mut Cursor<[u8; MAX_PAYLOAD_LENGTH]>, host: &str) -> EncodeDecodeResult<Self> {
        let mut username_length = [0u8; 4];
        let mut pid = [0u8; 4];
        let mut tty_length = [0u8; 4];
        let mut remote_tag = [0u8; 1];
        let mut remote_length = [0u8; 4];
        let mut active = [0u8; 1];
        let mut login_time = [0u8; 8];

        Session::read_field(cursor, &mut pid)?;
        Session::read_field(cursor, &mut login_time)?;

        Session::read_field(cursor, &mut username_length)?;
        let username_length = u32::from_be_bytes(username_length);
        if username_length as usize > MAX_USER_TTY_LENGTH {
            return Err(EncodeDecodeError::StringSizeLimitExceeded(username_length, MAX_USER_TTY_LENGTH));
        }

        let mut user = vec![0u8; username_length as usize];
        Session::read_field(cursor, &mut user)?;

        Session::read_field(cursor, &mut tty_length)?;
        let tty_length = u32::from_be_bytes(tty_length);
        if tty_length as usize > MAX_USER_TTY_LENGTH {
            return Err(EncodeDecodeError::StringSizeLimitExceeded(tty_length, MAX_USER_TTY_LENGTH));
        }

        let mut tty = vec![0u8; tty_length as usize];
        Session::read_field(cursor, &mut tty)?;

        Session::read_field(cursor, &mut remote_tag)?;
        if remote_tag[0] > 1 {
            return Err(EncodeDecodeError::NonbinaryBoolean);
        }

        let has_remote_tag = remote_tag[0] == 1;

        let remote = if has_remote_tag {
            Session::read_field(cursor, &mut remote_length)?;
            let remote_length = u32::from_be_bytes(remote_length);
            if remote_length as usize > MAX_USER_TTY_LENGTH {
                return Err(EncodeDecodeError::StringSizeLimitExceeded(username_length, MAX_USER_TTY_LENGTH));
            }

            if remote_length == 0 {
                return Err(EncodeDecodeError::EmptyRemote);
            }

            let mut remote = vec![0u8; remote_length as usize];
            Session::read_field(cursor, &mut remote)?;

            Some(String::from_utf8_lossy(&remote).to_string())
        } else {
            None
        };

        Session::read_field(cursor, &mut active)?;
        if active[0] > 1 {
            return Err(EncodeDecodeError::NonbinaryBoolean);
        }

        let user = String::from_utf8_lossy(&user).to_string();
        let pid = i32::from_be_bytes(pid);
        let tty = String::from_utf8_lossy(&tty).to_string();
        let active = active[0] == 1;
        let login_time = i64::from_be_bytes(login_time);

        let host = Some(host.to_string());

        Ok(Self {
            host,
            user,
            pid,
            tty,
            remote,
            active,
            login_time
        })
    }

    fn read_field(cursor: &mut Cursor<[u8; MAX_PAYLOAD_LENGTH]>, buffer: &mut [u8]) -> EncodeDecodeResult<()> {
        cursor.read_exact(buffer)?;
        Ok(())
    }

    /*fn read_field<T, F>(cursor: &mut Cursor<&[u8]>, convert_func: F) -> WhereResult<T>
    where
        N: const usize,
        F: FnOnce([u8; N]) -> EncodeDecodeError,
    {
        let mut buf = [0u8; N];
        cursor.read_exact(&mut buf)?;

        let value = convert_func(buf)?;
        Ok(value)
    }*/

    pub fn to_udp_payload(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        let pid = self.pid.to_be_bytes();
        let login_time = self.login_time.to_be_bytes();
        let user_length = (self.user.len() as u32).to_be_bytes();
        let user = self.user.as_bytes();
        let tty_length = (self.tty.len() as u32).to_be_bytes();
        let tty = self.tty.as_bytes();
        let active = self.active as u8;

        bytes.extend(&pid);
        bytes.extend(&login_time);
        bytes.extend(&user_length);
        bytes.extend(user);
        bytes.extend(&tty_length);
        bytes.extend(tty);

        match self.remote {
            None => bytes.push(0u8),
            Some(host) => {
                let host_bytes = host.into_bytes();
                let host_length = (host_bytes.len() as u32).to_be_bytes();

                bytes.extend(&host_length);
                bytes.extend(&host_bytes);
            }
        }

        bytes.push(active);

        bytes
    }
}

impl From<Utmpx> for Session {
    fn from(utmpx: Utmpx) -> Self {
        // BStr doesn't have a known size at compile time, so we can't use it instead of String
        let mut host = utmpx.host().to_string();
        host.truncate(MAX_REMOTE_LENGTH);

        let mut user = utmpx.user().to_string();
        user.truncate(MAX_USER_TTY_LENGTH);

        let pid = utmpx.process_id();
        // In the case of a user session, this will always be a TTY
        let mut tty = utmpx.device_name().to_string();
        tty.truncate(MAX_USER_TTY_LENGTH);

        let remote = if host.is_empty() {
            None
        } else {
            Some(host)
        };
        let active = utmpx.entry_type() == UtmpxKind::UserProcess;
        let login_time = utmpx.timeval().tv_sec;

        Self {
            host: None,
            user,
            pid,
            tty,
            remote,
            active,
            login_time
        }
    }
}
