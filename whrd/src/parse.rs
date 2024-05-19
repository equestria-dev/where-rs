use std::io::Read;

use crate::error::{EncodeDecodeError, WhereError, WhereResult};
use crate::PayloadCursor;

pub fn read_field<const N: usize, F, T>(cursor: &mut PayloadCursor, convert_func: F) -> WhereResult<T>
where
    F: Fn([u8; N]) -> WhereResult<T>
{
    let mut buffer = [0u8; N];
    cursor.read_exact(&mut buffer)?;

    let value = convert_func(buffer)?;
    Ok(value)
}

pub fn read_field_dynamic<F, T>(cursor: &mut PayloadCursor, size: usize, convert_func: F) -> WhereResult<T>
where
    F: Fn(Vec<u8>) -> WhereResult<T>
{
    let mut buffer = vec![0u8; size];
    cursor.read_exact(&mut buffer)?;

    let value = convert_func(buffer)?;
    Ok(value)
}

pub fn read_bool_field(cursor: &mut PayloadCursor) -> WhereResult<bool> {
    let value = read_field::<1, _, _>(cursor, |buf| Ok(buf[0] == 1))?;
    Ok(value)
}

pub fn read_string_field(cursor: &mut PayloadCursor, max_length: u32) -> WhereResult<String> {
    let string_length = read_field(cursor, |buf| Ok(u32::from_be_bytes(buf)))?;

    if string_length > max_length {
        return Err(WhereError::EncodeDecodeError(EncodeDecodeError::StringSizeLimitExceeded(string_length, max_length as usize)));
    }

    let string = read_field_dynamic(cursor, string_length as usize, |buf| Ok(String::from_utf8(buf)?))?;

    Ok(string)
}
