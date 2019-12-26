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
