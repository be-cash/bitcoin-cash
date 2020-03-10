use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;

pub fn read_var_int<R: io::Read>(read: &mut R) -> io::Result<u64> {
    let first_byte = read.read_u8()?;
    match first_byte {
        0..=0xfc => Ok(first_byte as u64),
        0xfd => Ok(read.read_u16::<LittleEndian>()? as u64),
        0xfe => Ok(read.read_u32::<LittleEndian>()? as u64),
        0xff => Ok(read.read_u64::<LittleEndian>()? as u64),
    }
}

pub fn write_var_int<W: io::Write>(write: &mut W, number: u64) -> io::Result<()> {
    match number {
        0..=0xfc => write.write_u8(number as u8)?,
        0xfd..=0xffff => {
            write.write_all(b"\xfd")?;
            write.write_u16::<LittleEndian>(number as u16)?
        }
        0x10000..=0xffff_ffff => {
            write.write_all(b"\xfe")?;
            write.write_u32::<LittleEndian>(number as u32)?
        }
        _ => {
            write.write_all(b"\xff")?;
            write.write_u64::<LittleEndian>(number as u64)?
        }
    }
    Ok(())
}

pub fn encode_minimally(vec: &mut Vec<u8>) {
    // If the last byte is not 0x00 or 0x80, we are minimally encoded.
    if let Some(&last) = vec.last() {
        if last & 0x7f != 0 {
            return;
        }
        // If the script is one byte long, then we have a zero, which encodes as an
        // empty array.
        if vec.len() == 1 {
            vec.clear();
            return;
        }
        // If the next byte has it sign bit set, then we are minimally encoded.
        if vec[vec.len() - 2] & 0x80 != 0 {
            return;
        }
        // We are not minimally encoded, we need to figure out how much to trim.
        let mut i = vec.len() - 1;
        while i > 0 {
            // We found a non zero byte, time to encode.
            if vec[i - 1] != 0 {
                if vec[i - 1] & 0x80 != 0 {
                    // We found a byte with it sign bit set so we need one more byte.
                    vec[i] = last;
                    i += 1;
                } else {
                    // the sign bit is clear, we can use it.
                    vec[i - 1] |= last;
                }
                vec.resize(i, 0u8);
                return;
            }
            i -= 1;
        }
        vec.resize(i, 0u8);
    }
}

pub fn encode_int(int: i32) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.write_i32::<LittleEndian>(int.abs()).unwrap();
    if int < 0 {
        vec.write_u8(0x80).unwrap();
    }
    encode_minimally(&mut vec);
    vec
}

pub fn encode_bool(b: bool) -> Vec<u8> {
    if b {
        vec![0x01]
    } else {
        vec![]
    }
}

pub fn vec_to_int(vec: &[u8]) -> i32 {
    if vec.is_empty() {
        return 0;
    }
    let mut shift = 0;
    let mut int = 0;
    let sign_bit = vec[vec.len() - 1] & 0x80;
    for (i, value) in vec.iter().enumerate() {
        if i == vec.len() - 1 && sign_bit != 0 {
            int += ((*value ^ sign_bit) as i32) << (shift);
            int *= -1;
        } else {
            int += (*value as i32) << (shift);
            shift += 8;
        }
    }
    int
}
