use std::io::Cursor;
use coreutils_core::os::utmpx::*;

use crate::error::{WhereResult, EncodeDecodeResult, EncodeDecodeError};

mod parse;
pub mod error;

pub const WHERED_MAGIC: [u8; 4] = *b"WHRD";
pub const MAX_USER_TTY_LENGTH: usize = 32;
pub const MAX_REMOTE_LENGTH: usize = 64;
pub const MAX_ENTRY_LENGTH: usize = MAX_REMOTE_LENGTH + MAX_USER_TTY_LENGTH * 2 + 25;
pub const MAX_PAYLOAD_LENGTH: usize = 65501;
pub const MAX_PAYLOAD_ENTRIES: usize = MAX_PAYLOAD_LENGTH / MAX_ENTRY_LENGTH;

type Payload = [u8; MAX_PAYLOAD_LENGTH];
type PayloadCursor = Cursor<Payload>;

#[derive(Debug)]
pub struct Session {
    pid: i32,
    login_time: i64,
    user: String,
    tty: String,
    remote: Option<String>,
    active: bool,
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

    pub fn from_udp_payload(buffer: Payload) -> WhereResult<Self> {
        let mut cursor = Cursor::new(buffer);
        let mut inner = vec![];

        // Check magic
        parse::read_field(&mut cursor, |buf| {
            if buf != WHERED_MAGIC {
                Err(EncodeDecodeError::BadMagic(buf))?
            } else {
                Ok(())
            }
        })?;

        let entry_count = parse::read_field(&mut cursor, |buf| Ok(u32::from_be_bytes(buf)))?;

        for _ in 0..entry_count {
            inner.push(Session::from_udp_payload(&mut cursor)?);
        }

        Ok(Self {
            inner
        })
    }
}

impl Session {
    pub fn from_udp_payload(cursor: &mut PayloadCursor) -> WhereResult<Self> {
        let pid = parse::read_field(cursor, |buf| Ok(i32::from_be_bytes(buf)))?;
        let login_time = parse::read_field(cursor, |buf| Ok(i64::from_be_bytes(buf)))?;
        let user = parse::read_string_field(cursor)?;
        let tty = parse::read_string_field(cursor)?;

        let remote = {
            let has_remote_tag = parse::read_bool_field(cursor)?;
            if has_remote_tag {
                Some(parse::read_string_field(cursor)?)
            } else {
                None
            }
        };

        let active = parse::read_bool_field(cursor)?;

        Ok(Self {
            pid,
            login_time,
            user,
            tty,
            remote,
            active,
        })
    }

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
            user,
            pid,
            tty,
            remote,
            active,
            login_time
        }
    }
}
