use byteorder::{LittleEndian, WriteBytesExt};

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
