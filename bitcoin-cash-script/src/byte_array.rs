use crate::Integer;
use byteorder::{LittleEndian, WriteBytesExt};
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Function {
    Plain,
    Sha1,
    Ripemd160,
    Sha256,
    Hash256,
    Hash160,
    Num2Bin,
    EcdsaSign,
    SchnorrSign,
    ToDataSig,
    UnexpectedSplit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteArray<'a> {
    pub data: Cow<'a, [u8]>,
    pub function: Function,
    pub preimage: Option<Vec<ByteArray<'a>>>,
}

impl<'a> ByteArray<'a> {
    pub fn new(data: Cow<'a, [u8]>) -> Self {
        ByteArray {
            data,
            function: Function::Plain,
            preimage: None,
        }
    }

    pub fn from_slice(slice: &'a [u8]) -> Self {
        ByteArray {
            data: slice.into(),
            function: Function::Plain,
            preimage: None,
        }
    }

    pub fn concat(self, other: ByteArray<'a>) -> ByteArray<'a> {
        let mut new_data = Vec::with_capacity(self.data.len() + other.data.len());
        new_data.extend_from_slice(&self.data);
        new_data.extend_from_slice(&other.data);
        if self.function == Function::Plain && other.function == Function::Plain {
            let mut new_preimage = if self.preimage.is_some() {
                self.preimage.unwrap()
            } else {
                vec![self]
            };
            new_preimage.append(&mut if other.preimage.is_some() {
                other.preimage.unwrap()
            } else {
                vec![other]
            });
            return ByteArray {
                data: new_data.into(),
                function: Function::Plain,
                preimage: Some(new_preimage.into()),
            };
        }
        ByteArray {
            data: new_data.into(),
            function: Function::Plain,
            preimage: Some(vec![self, other].into()),
        }
    }

    pub fn split(self, at: usize) -> Result<(ByteArray<'a>, ByteArray<'a>), String> {
        if self.data.len() < at {
            return Err(format!(
                "Index {} is out of bounds for array with length {}, {}.",
                at,
                self.data.len(),
                hex::encode(&self.data)
            ));
        }
        let mut data = self.data.into_owned();
        let other = data.split_off(at);
        let mut function = Function::Plain;
        let (left_preimage, right_preimage) = match self.preimage {
            Some(preimage) if self.function == Function::Plain => {
                let mut left_preimage = Vec::new();
                let mut right_preimage = Vec::new();
                let mut is_left = true;
                let mut len = 0;
                for part in preimage {
                    let part_len = part.data.len();
                    if len == at {
                        is_left = false;
                    }
                    if len + part_len > at && is_left {
                        let part_function = part.function;
                        let (mut sub_left, mut sub_right) = part.split(at - len)?;
                        if part_function != Function::Plain {
                            sub_left.function = Function::UnexpectedSplit;
                            sub_right.function = Function::UnexpectedSplit;
                        }
                        left_preimage.push(sub_left);
                        right_preimage.push(sub_right);
                        is_left = false;
                    } else {
                        if is_left {
                            left_preimage.push(part);
                        } else {
                            right_preimage.push(part);
                        }
                    }
                    len += part_len;
                }
                (Some(left_preimage), Some(right_preimage))
            }
            _ => {
                function = Function::UnexpectedSplit;
                (None, None)
            }
        };
        Ok((
            ByteArray {
                data: data.into(),
                function: function,
                preimage: left_preimage.map(Into::into),
            },
            ByteArray {
                data: other.into(),
                function: function,
                preimage: right_preimage.map(Into::into),
            },
        ))
    }

    pub fn apply_function(self, data: Cow<'a, [u8]>, function: Function) -> ByteArray<'a> {
        ByteArray {
            data,
            function,
            preimage: if self.preimage.is_some() {
                self.preimage
            } else {
                Some(vec![self].into())
            },
        }
    }

    pub fn from_int(int: Integer, n_bytes: Integer) -> Result<Self, String> {
        if n_bytes <= 0 {
            return Err(format!("n_bytes={} not valid", n_bytes));
        }
        let max_bits = (n_bytes * 8 - 1).min(31) as u128;
        let max_num: u128 = 1 << max_bits;
        let max_num = (max_num - 1) as Integer;
        let min_num = -max_num;
        if int < min_num || int > max_num {
            return Err(format!("int={} not valid for n_bytes={}", int, n_bytes));
        }
        let mut bytes = Vec::new();
        bytes.write_i32::<LittleEndian>(int.abs()).unwrap();
        let n_bytes = n_bytes as usize;
        if bytes.len() < n_bytes {
            bytes.append(&mut vec![0; n_bytes - bytes.len()]);
        } else if bytes.len() > n_bytes {
            bytes.drain(n_bytes..);
        }
        if int < 0 {
            let len = bytes.len();
            bytes[len - 1] |= 0x80;
        }
        Ok(ByteArray {
            data: bytes.into(),
            function: Function::Num2Bin,
            preimage: None,
        })
    }

    pub fn to_owned_array(&self) -> ByteArray<'static> {
        ByteArray {
            data: self.data.clone().into_owned().into(),
            function: self.function,
            preimage: self.preimage.as_ref().map(|preimage| {
                preimage
                    .iter()
                    .cloned()
                    .map(|part| part.to_owned_array())
                    .collect()
            }),
        }
    }
}

impl Default for ByteArray<'_> {
    fn default() -> Self {
        ByteArray::from_slice(&[])
    }
}

impl From<Vec<u8>> for ByteArray<'static> {
    fn from(vec: Vec<u8>) -> Self {
        ByteArray::new(vec.into())
    }
}

impl<'a> From<Cow<'a, [u8]>> for ByteArray<'a> {
    fn from(cow: Cow<'a, [u8]>) -> Self {
        ByteArray::new(cow)
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteArray, Function};
    use sha2::Digest;

    #[test]
    fn test_cat() {
        let a = ByteArray::from_slice(b"A");
        let b = ByteArray::from_slice(b"B");
        let c = ByteArray::from_slice(b"C");
        let d = ByteArray::from_slice(b"D");
        let ab = a.concat(b);
        {
            assert_eq!(ab.data.as_ref(), b"AB");
            let preimage = ab.preimage.as_ref().expect("No preimage");
            assert_eq!(preimage[0].data.as_ref(), b"A");
            assert_eq!(preimage[0].preimage, None);
            assert_eq!(preimage[1].data.as_ref(), b"B");
            assert_eq!(preimage[1].preimage, None);
        }
        let abcd = ab.concat(c.concat(d));
        {
            assert_eq!(abcd.data.as_ref(), b"ABCD");
            let preimage = abcd.preimage.as_ref().expect("No preimage");
            assert_eq!(preimage.len(), 4);
            assert_eq!(preimage[0].data.as_ref(), b"A");
            assert_eq!(preimage[0].preimage, None);
            assert_eq!(preimage[1].data.as_ref(), b"B");
            assert_eq!(preimage[1].preimage, None);
            assert_eq!(preimage[2].data.as_ref(), b"C");
            assert_eq!(preimage[2].preimage, None);
            assert_eq!(preimage[3].data.as_ref(), b"D");
            assert_eq!(preimage[3].preimage, None);
        }
    }

    #[test]
    fn test_hash() {
        let a = ByteArray::from_slice(b"A");
        let b = ByteArray::from_slice(b"B");
        let c = ByteArray::from_slice(b"C");
        let cat = a.concat(b).concat(c);
        let hash = sha2::Sha256::digest(&cat.data);
        let hashed = cat.apply_function(hash.as_ref().into(), Function::Sha256);
        let preimage = hashed.preimage.as_ref().expect("No preimage");
        assert_eq!(hashed.data.as_ref(), hash.as_ref());
        assert_eq!(preimage[0].data.as_ref(), b"A");
        assert_eq!(preimage[0].preimage, None);
        assert_eq!(preimage[1].data.as_ref(), b"B");
        assert_eq!(preimage[1].preimage, None);
        assert_eq!(preimage[2].data.as_ref(), b"C");
        assert_eq!(preimage[2].preimage, None);
    }

    #[test]
    fn test_hash_nested() {
        let a = ByteArray::from_slice(b"A");
        let b = ByteArray::from_slice(b"B");
        let inner = a.concat(b);
        let inner_hash = sha2::Sha256::digest(&inner.data);
        let inner_hashed = inner.apply_function(inner_hash.as_ref().into(), Function::Sha256);
        let c = ByteArray::from_slice(b"C");
        let d = ByteArray::from_slice(b"D");
        let outer = c.concat(inner_hashed).concat(d);
        let outer_hash = sha2::Sha256::digest(&outer.data);
        let outer_hashed = outer.apply_function(outer_hash.as_ref().into(), Function::Sha256);
        assert_eq!(outer_hashed.data.as_ref(), outer_hash.as_ref());
        let outer_preimage = outer_hashed.preimage.as_ref().expect("No preimage");
        assert_eq!(outer_preimage.len(), 3);
        assert_eq!(outer_preimage[0].data.as_ref(), b"C");
        assert_eq!(outer_preimage[0].preimage, None);
        assert_eq!(outer_preimage[1].data.as_ref(), inner_hash.as_ref());
        assert!(outer_preimage[1].preimage.is_some());
        assert_eq!(outer_preimage[2].data.as_ref(), b"D");
        assert_eq!(outer_preimage[2].preimage, None);

        let inner_preimage = outer_preimage[1].preimage.as_ref().expect("No preimage");
        assert_eq!(inner_preimage.len(), 2);
        assert_eq!(inner_preimage[0].data.as_ref(), b"A");
        assert_eq!(inner_preimage[0].preimage, None);
        assert_eq!(inner_preimage[1].data.as_ref(), b"B");
        assert_eq!(inner_preimage[1].preimage, None);
    }

    #[test]
    fn test_split_a_b() {
        let a = ByteArray::from_slice(b"A");
        let b = ByteArray::from_slice(b"B");
        let cat = a.concat(b);
        let (left, right) = cat.split(1).unwrap();
        let left_preimage = left.preimage.as_ref().expect("No preimage");
        let right_preimage = right.preimage.as_ref().expect("No preimage");
        assert_eq!(left.function, Function::Plain);
        assert_eq!(left.data.as_ref(), b"A");
        assert_eq!(left_preimage.len(), 1);
        assert_eq!(left_preimage[0].data.as_ref(), b"A");
        assert_eq!(right.function, Function::Plain);
        assert_eq!(right.data.as_ref(), b"B");
        assert_eq!(right_preimage.len(), 1);
        assert_eq!(right_preimage[0].data.as_ref(), b"B");
    }

    #[test]
    fn test_split_nested() {
        let a = ByteArray::from_slice(b"A");
        let b = ByteArray::from_slice(b"B");
        let inner = a.concat(b);
        let inner_hash = sha2::Sha256::digest(&inner.data);
        let inner_hashed = inner.apply_function(inner_hash.as_ref().into(), Function::Sha256);
        let c = ByteArray::from_slice(b"C");
        let d = ByteArray::from_slice(b"D");
        let outer = c.concat(inner_hashed.clone()).concat(d);

        // test 1, split neatly at 1
        {
            let (left, right) = outer.clone().split(1).unwrap();
            let left_preimage = left.preimage.as_ref().expect("No preimage");
            let right_preimage = right.preimage.as_ref().expect("No preimage");
            assert_eq!(left.function, Function::Plain);
            assert_eq!(left.data.as_ref(), b"C");
            assert_eq!(left_preimage.len(), 1);
            assert_eq!(left_preimage[0].data.as_ref(), b"C");
            assert_eq!(left_preimage[0].preimage, None);
            assert_eq!(right.function, Function::Plain);
            assert_eq!(
                right.data.as_ref(),
                [inner_hash.as_ref(), b"D"].concat().as_slice()
            );
            assert_eq!(right_preimage.len(), 2);
            assert_eq!(right_preimage[0].function, Function::Sha256);
            assert_eq!(right_preimage[0].data.as_ref(), inner_hash.as_ref());
            assert_eq!(right_preimage[1].function, Function::Plain);
            assert_eq!(right_preimage[1].data.as_ref(), b"D");
            let right_preimage2 = right_preimage[0].preimage.as_ref().expect("No preimage");
            assert_eq!(right_preimage2[0].function, Function::Plain);
            assert_eq!(right_preimage2[0].data.as_ref(), b"A");
            assert_eq!(right_preimage2[0].preimage, None);
            assert_eq!(right_preimage2[1].function, Function::Plain);
            assert_eq!(right_preimage2[1].data.as_ref(), b"B");
            assert_eq!(right_preimage2[1].preimage, None);
        }

        // test 2, split in middle of hash
        {
            let (left, right) = outer.clone().split(3).unwrap();
            let left_preimage = left.preimage.as_ref().expect("No preimage");
            let right_preimage = right.preimage.as_ref().expect("No preimage");
            assert_eq!(left.function, Function::Plain);
            assert_eq!(
                left.data.as_ref(),
                [b"C", &inner_hash[..2]].concat().as_slice()
            );
            assert_eq!(left_preimage.len(), 2);
            assert_eq!(left_preimage[0].function, Function::Plain);
            assert_eq!(left_preimage[0].data.as_ref(), b"C");
            assert_eq!(left_preimage[0].preimage, None);
            assert_eq!(left_preimage[1].function, Function::UnexpectedSplit);
            assert_eq!(left_preimage[1].data.as_ref(), &inner_hash[..2]);
            assert_eq!(left_preimage[1].preimage, None);
            assert_eq!(right.function, Function::Plain);
            assert_eq!(
                right.data.as_ref(),
                [&inner_hash[2..], b"D"].concat().as_slice()
            );
            assert_eq!(right_preimage[0].data.as_ref(), &inner_hash[2..]);
            assert_eq!(right_preimage[0].preimage, None);
            assert_eq!(right_preimage[1].data.as_ref(), b"D");
            assert_eq!(right_preimage[1].preimage, None);
        }

        // test 3, split neatly after hash
        {
            let (left, right) = outer.clone().split(33).unwrap();
            let left_preimage = left.preimage.as_ref().expect("No preimage");
            let right_preimage = right.preimage.as_ref().expect("No preimage");
            assert_eq!(left.function, Function::Plain);
            assert_eq!(
                left.data.as_ref(),
                [b"C", inner_hash.as_ref()].concat().as_slice()
            );
            assert_eq!(left_preimage.len(), 2);
            assert_eq!(left_preimage[0].function, Function::Plain);
            assert_eq!(left_preimage[0].data.as_ref(), b"C");
            assert_eq!(left_preimage[0].preimage, None);
            assert_eq!(left_preimage[1].function, Function::Sha256);
            assert_eq!(left_preimage[1].data.as_ref(), inner_hash.as_ref());
            assert_eq!(&left_preimage[1], &inner_hashed);
            assert_eq!(right.function, Function::Plain);
            assert_eq!(right.data.as_ref(), b"D");
            assert_eq!(right_preimage[0].data.as_ref(), b"D");
            assert_eq!(right_preimage[0].preimage, None);
        }
    }
}
