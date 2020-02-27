use num_derive::*;

use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::data_type::{BitcoinBoolean, BitcoinByteArray, BitcoinInteger, DataType};

lazy_static! {
    pub static ref MAP_NAME_TO_ENUM: HashMap<String, OpcodeType> = {
        let mut map = HashMap::new();
        map.insert("OP_0".to_string(), OpcodeType::OP_0);
        for code in 0x51..OpcodeType::FIRST_UNDEFINED_OP_VALUE as u8 {
            let opcode: OpcodeType =
                num::FromPrimitive::from_u8(code).expect(&format!("Invalid opcode {}", code));
            map.insert(format!("{:?}", opcode), opcode);
        }
        map
    };
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum StackItemDelta {
    Untouched,
    Added,
    Changed,
    Moved,
    MovedIndirectly,
    Observed,
    Removed,
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct OpcodeBehavior {
    pub input_types: &'static [DataType],
    pub output_types: &'static [DataType],
    pub output_order: Option<&'static [usize]>,
    pub delta: &'static [StackItemDelta],
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, FromPrimitive)]
pub enum OpcodeType {
    // push value
    OP_0 = 0x00,
    OP_PUSHDATA1 = 0x4c,
    OP_PUSHDATA2 = 0x4d,
    OP_PUSHDATA4 = 0x4e,
    OP_1NEGATE = 0x4f,
    OP_RESERVED = 0x50,
    OP_1 = 0x51,
    OP_2 = 0x52,
    OP_3 = 0x53,
    OP_4 = 0x54,
    OP_5 = 0x55,
    OP_6 = 0x56,
    OP_7 = 0x57,
    OP_8 = 0x58,
    OP_9 = 0x59,
    OP_10 = 0x5a,
    OP_11 = 0x5b,
    OP_12 = 0x5c,
    OP_13 = 0x5d,
    OP_14 = 0x5e,
    OP_15 = 0x5f,
    OP_16 = 0x60,

    // control
    OP_NOP = 0x61,
    OP_VER = 0x62,
    OP_IF = 0x63,
    OP_NOTIF = 0x64,
    OP_VERIF = 0x65,
    OP_VERNOTIF = 0x66,
    OP_ELSE = 0x67,
    OP_ENDIF = 0x68,
    OP_VERIFY = 0x69,
    OP_RETURN = 0x6a,

    // stack ops
    OP_TOALTSTACK = 0x6b,
    OP_FROMALTSTACK = 0x6c,
    OP_2DROP = 0x6d,
    OP_2DUP = 0x6e,
    OP_3DUP = 0x6f,
    OP_2OVER = 0x70,
    OP_2ROT = 0x71,
    OP_2SWAP = 0x72,
    OP_IFDUP = 0x73,
    OP_DEPTH = 0x74,
    OP_DROP = 0x75,
    OP_DUP = 0x76,
    OP_NIP = 0x77,
    OP_OVER = 0x78,
    OP_PICK = 0x79,
    OP_ROLL = 0x7a,
    OP_ROT = 0x7b,
    OP_SWAP = 0x7c,
    OP_TUCK = 0x7d,

    // splice ops
    OP_CAT = 0x7e,
    OP_SPLIT = 0x7f,   // after monolith upgrade (May 2018)
    OP_NUM2BIN = 0x80, // after monolith upgrade (May 2018)
    OP_BIN2NUM = 0x81, // after monolith upgrade (May 2018)
    OP_SIZE = 0x82,

    // bit logic
    OP_INVERT = 0x83,
    OP_AND = 0x84,
    OP_OR = 0x85,
    OP_XOR = 0x86,
    OP_EQUAL = 0x87,
    OP_EQUALVERIFY = 0x88,
    OP_RESERVED1 = 0x89,
    OP_RESERVED2 = 0x8a,

    // numeric
    OP_1ADD = 0x8b,
    OP_1SUB = 0x8c,
    OP_2MUL = 0x8d,
    OP_2DIV = 0x8e,
    OP_NEGATE = 0x8f,
    OP_ABS = 0x90,
    OP_NOT = 0x91,
    OP_0NOTEQUAL = 0x92,

    OP_ADD = 0x93,
    OP_SUB = 0x94,
    OP_MUL = 0x95,
    OP_DIV = 0x96,
    OP_MOD = 0x97,
    OP_LSHIFT = 0x98,
    OP_RSHIFT = 0x99,

    OP_BOOLAND = 0x9a,
    OP_BOOLOR = 0x9b,
    OP_NUMEQUAL = 0x9c,
    OP_NUMEQUALVERIFY = 0x9d,
    OP_NUMNOTEQUAL = 0x9e,
    OP_LESSTHAN = 0x9f,
    OP_GREATERTHAN = 0xa0,
    OP_LESSTHANOREQUAL = 0xa1,
    OP_GREATERTHANOREQUAL = 0xa2,
    OP_MIN = 0xa3,
    OP_MAX = 0xa4,

    OP_WITHIN = 0xa5,

    // crypto
    OP_RIPEMD160 = 0xa6,
    OP_SHA1 = 0xa7,
    OP_SHA256 = 0xa8,
    OP_HASH160 = 0xa9,
    OP_HASH256 = 0xaa,
    OP_CODESEPARATOR = 0xab,
    OP_CHECKSIG = 0xac,
    OP_CHECKSIGVERIFY = 0xad,
    OP_CHECKMULTISIG = 0xae,
    OP_CHECKMULTISIGVERIFY = 0xaf,

    // expansion
    OP_NOP1 = 0xb0,
    OP_CHECKLOCKTIMEVERIFY = 0xb1,
    OP_CHECKSEQUENCEVERIFY = 0xb2,
    OP_NOP4 = 0xb3,
    OP_NOP5 = 0xb4,
    OP_NOP6 = 0xb5,
    OP_NOP7 = 0xb6,
    OP_NOP8 = 0xb7,
    OP_NOP9 = 0xb8,
    OP_NOP10 = 0xb9,

    // More crypto
    OP_CHECKDATASIG = 0xba,
    OP_CHECKDATASIGVERIFY = 0xbb,

    // The first op_code value after all defined opcodes
    FIRST_UNDEFINED_OP_VALUE,

    // multi-byte opcodes
    OP_PREFIX_BEGIN = 0xf0,
    OP_PREFIX_END = 0xf7,

    OP_INVALIDOPCODE = 0xff,
}

pub mod func {
    #![allow(non_snake_case)]
    #![allow(unused_variables)]

    use super::*;

    pub fn SECOND<T>(item1: T, item2: T) -> T {
        item2
    }

    pub fn OP_1NEGATE() -> BitcoinInteger {
        BitcoinInteger(-1)
    }
    pub fn OP_0() -> BitcoinInteger {
        BitcoinInteger(0)
    }
    pub fn OP_1() -> BitcoinInteger {
        BitcoinInteger(1)
    }
    pub fn OP_2() -> BitcoinInteger {
        BitcoinInteger(2)
    }
    pub fn OP_3() -> BitcoinInteger {
        BitcoinInteger(3)
    }
    pub fn OP_4() -> BitcoinInteger {
        BitcoinInteger(4)
    }
    pub fn OP_5() -> BitcoinInteger {
        BitcoinInteger(5)
    }
    pub fn OP_6() -> BitcoinInteger {
        BitcoinInteger(6)
    }
    pub fn OP_7() -> BitcoinInteger {
        BitcoinInteger(7)
    }
    pub fn OP_8() -> BitcoinInteger {
        BitcoinInteger(8)
    }
    pub fn OP_9() -> BitcoinInteger {
        BitcoinInteger(9)
    }
    pub fn OP_10() -> BitcoinInteger {
        BitcoinInteger(10)
    }
    pub fn OP_11() -> BitcoinInteger {
        BitcoinInteger(11)
    }
    pub fn OP_12() -> BitcoinInteger {
        BitcoinInteger(12)
    }
    pub fn OP_13() -> BitcoinInteger {
        BitcoinInteger(13)
    }
    pub fn OP_14() -> BitcoinInteger {
        BitcoinInteger(14)
    }
    pub fn OP_15() -> BitcoinInteger {
        BitcoinInteger(15)
    }
    pub fn OP_16() -> BitcoinInteger {
        BitcoinInteger(16)
    }

    pub fn OP_IF<T>(stack_item: T) {}
    pub fn OP_ELSE() {}
    pub fn OP_ENDIF() {}
    pub fn OP_VERIFY<T>(stack_item: T) {}

    pub fn OP_TOALTSTACK<T>(stack_item: T) -> T {
        stack_item
    }
    pub fn OP_FROMALTSTACK<T>(alt_stack_item: T) -> T {
        alt_stack_item
    }
    pub fn OP_2DROP<T, U>(item1: T, item2: U) {}
    pub fn OP_2DUP<T: Clone, U: Clone>(item1: T, item2: U) -> (T, U, T, U) {
        (item1.clone(), item2.clone(), item1, item2)
    }
    pub fn OP_3DUP<T: Clone, U: Clone, V: Clone>(
        item1: T,
        item2: U,
        item3: V,
    ) -> (T, U, V, T, U, V) {
        (
            item1.clone(),
            item2.clone(),
            item3.clone(),
            item1,
            item2,
            item3,
        )
    }
    pub fn OP_2OVER<T: Clone, U: Clone, V, W>(
        item1: T,
        item2: U,
        item3: V,
        item4: W,
    ) -> (T, U, V, W, T, U) {
        (item1.clone(), item2.clone(), item3, item4, item1, item2)
    }
    pub fn OP_2ROT<T, U, V, W, X, Y>(
        item1: T,
        item2: U,
        item3: V,
        item4: W,
        item5: X,
        item6: Y,
    ) -> (V, W, X, Y, T, U) {
        (item3, item4, item5, item6, item1, item2)
    }
    pub fn OP_2SWAP<T, U, V, W>(item1: T, item2: U, item3: V, item4: W) -> (V, W, T, U) {
        (item3, item4, item1, item2)
    }
    pub fn OP_DEPTH() -> BitcoinInteger {
        BitcoinInteger(0x10d0_10d0)
    }
    pub fn OP_DROP<T>(item: T) {}
    pub fn OP_DUP<T: Clone>(item: T) -> (T, T) {
        (item.clone(), item)
    }
    pub fn OP_NIP<T, U>(item1: T, item2: U) -> U {
        item2
    }
    pub fn OP_OVER<T: Clone, U>(item1: T, item2: U) -> (T, U, T) {
        (item1.clone(), item2, item1)
    }
    pub fn OP_PICK(depth: BitcoinInteger) {}
    pub fn OP_ROLL(depth: BitcoinInteger) {}
    pub fn OP_ROT<T, U, V>(item1: T, item2: U, item3: V) -> (U, V, T) {
        (item2, item3, item1)
    }
    pub fn OP_SWAP<T, U>(item1: T, item2: U) -> (U, T) {
        (item2, item1)
    }
    pub fn OP_TUCK<T, U: Clone>(item1: T, item2: U) -> (U, T, U) {
        (item2.clone(), item1, item2)
    }

    pub fn OP_CAT(left: BitcoinByteArray, right: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray([&left.0[..], &right.0[..]][..].concat())
    }
    pub fn OP_SPLIT(
        array: BitcoinByteArray,
        split_idx: BitcoinInteger,
    ) -> (BitcoinByteArray, BitcoinByteArray) {
        (
            BitcoinByteArray(b"left".to_vec()),
            BitcoinByteArray(b"right".to_vec()),
        )
    }
    pub fn OP_NUM2BIN(num: BitcoinInteger, byte_size: BitcoinInteger) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_BIN2NUM(array: BitcoinByteArray) -> BitcoinInteger {
        BitcoinInteger(0x10d0_10d0)
    }
    pub fn OP_SIZE(array: BitcoinByteArray) -> (BitcoinByteArray, BitcoinInteger) {
        let size = BitcoinInteger(array.0.len() as i32);
        (array, size)
    }
    pub fn OP_AND(array1: BitcoinByteArray, array2: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_OR(array1: BitcoinByteArray, array2: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_XOR(array1: BitcoinByteArray, array2: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_EQUAL<T: Eq>(item1: T, item2: T) -> BitcoinBoolean {
        BitcoinBoolean(item1 == item2)
    }
    pub fn OP_EQUALVERIFY<T: Eq>(item1: T, item2: T) {}

    pub fn OP_1ADD(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(num.0 + 1)
    }
    pub fn OP_1SUB(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(num.0 - 1)
    }
    pub fn OP_NEGATE(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(-num.0)
    }
    pub fn OP_ABS(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(num.0.abs())
    }
    pub fn OP_NOT(boolean: BitcoinBoolean) -> BitcoinBoolean {
        BitcoinBoolean(!boolean.0)
    }
    pub fn OP_0NOTEQUAL(num: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num.0 != 0)
    }
    pub fn OP_ADD(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(num1.0 + num2.0)
    }
    pub fn OP_SUB(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(num1.0 - num2.0)
    }
    pub fn OP_DIV(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(0x10d0_10d0)
    }
    pub fn OP_MOD(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(0x10d0_10d0)
    }
    pub fn OP_BOOLAND(boolean1: BitcoinBoolean, boolean2: BitcoinBoolean) -> BitcoinBoolean {
        BitcoinBoolean(boolean1.0 && boolean2.0)
    }
    pub fn OP_BOOLOR(boolean1: BitcoinBoolean, boolean2: BitcoinBoolean) -> BitcoinBoolean {
        BitcoinBoolean(boolean1.0 && boolean2.0)
    }
    pub fn OP_NUMEQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 == num2.0)
    }
    pub fn OP_NUMEQUALVERIFY(num1: BitcoinInteger, num2: BitcoinInteger) {}
    pub fn OP_NUMNOTEQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 != num2.0)
    }
    pub fn OP_LESSTHAN(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 < num2.0)
    }
    pub fn OP_GREATERTHAN(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 > num2.0)
    }
    pub fn OP_LESSTHANOREQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 <= num2.0)
    }
    pub fn OP_GREATERTHANOREQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 >= num2.0)
    }
    pub fn OP_MIN() {}
    pub fn OP_MAX() {}
    pub fn OP_WITHIN(
        num1: BitcoinInteger,
        num_min: BitcoinInteger,
        num_max: BitcoinInteger,
    ) -> BitcoinBoolean {
        BitcoinBoolean(num1.0 >= num_min.0 && num1.0 <= num_max.0)
    }
    pub fn OP_RIPEMD160(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_SHA1(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_SHA256(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_HASH160(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_HASH256(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"TODO".to_vec())
    }
    pub fn OP_CODESEPARATOR() {}
    pub fn OP_CHECKSIG(sig: BitcoinByteArray, pubkey: BitcoinByteArray) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_CHECKSIGVERIFY(sig: BitcoinByteArray, pubkey: BitcoinByteArray) {}
    pub fn OP_CHECKLOCKTIMEVERIFY(locktime: BitcoinByteArray) -> BitcoinByteArray {
        locktime
    }
    pub fn OP_CHECKSEQUENCEVERIFY(sequence: BitcoinByteArray) -> BitcoinByteArray {
        sequence
    }
    pub fn OP_CHECKDATASIG(
        sig: BitcoinByteArray,
        data: BitcoinByteArray,
        pubkey: BitcoinByteArray,
    ) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_CHECKDATASIGVERIFY(
        sig: BitcoinByteArray,
        data: BitcoinByteArray,
        pubkey: BitcoinByteArray,
    ) {
    }
}

impl OpcodeType {
    pub fn is_disabled(self) -> bool {
        use OpcodeType::*;
        match self {
            OP_RESERVED | OP_RESERVED1 | OP_RESERVED2 | OP_MUL | OP_2MUL | OP_2DIV | OP_INVERT
            | OP_LSHIFT | OP_RSHIFT | OP_VER | OP_VERIF | OP_VERNOTIF => true,
            opcode => opcode as u8 >= FIRST_UNDEFINED_OP_VALUE as u8,
        }
    }

    pub fn behavior(self) -> OpcodeBehavior {
        use DataType::*;
        use OpcodeType::*;
        use StackItemDelta::*;
        fn o(
            input_types: &'static [DataType],
            output_types: &'static [DataType],
            output_order: &'static [usize],
            delta: &'static [StackItemDelta],
        ) -> OpcodeBehavior {
            OpcodeBehavior {
                input_types,
                output_types,
                output_order: Some(output_order),
                delta,
            }
        }
        fn u(
            input_types: &'static [DataType],
            output_types: &'static [DataType],
            delta: &'static [StackItemDelta],
        ) -> OpcodeBehavior {
            OpcodeBehavior {
                input_types,
                output_types,
                output_order: None,
                delta,
            }
        }
        const T: DataType = Generic;
        match self {
            OP_VERIFY => u(&[Boolean], &[], &[]),

            OP_2DROP => o(&[T, T], &[], &[], &[]),
            OP_2DUP => o(
                &[T, T],
                &[T, T, T, T],
                &[0, 1, 0, 1],
                &[Observed, Observed, Added, Added],
            ),
            OP_3DUP => o(
                &[T, T, T],
                &[T, T, T, T, T, T],
                &[0, 1, 2, 0, 1, 2],
                &[Observed, Observed, Observed, Added, Added, Added],
            ),
            OP_2OVER => o(
                &[T, T, T, T],
                &[T, T, T, T, T, T],
                &[0, 1, 2, 3, 0, 1],
                &[
                    Observed,
                    Observed,
                    Untouched,
                    Untouched,
                    Added,
                    Added,
                ],
            ),
            OP_2ROT => o(
                &[T, T, T, T, T, T],
                &[T, T, T, T, T, T],
                &[2, 3, 4, 5, 0, 1],
                &[
                    MovedIndirectly,
                    MovedIndirectly,
                    MovedIndirectly,
                    MovedIndirectly,
                    Moved,
                    Moved,
                ],
            ),
            OP_2SWAP => o(
                &[T, T, T, T],
                &[T, T, T, T],
                &[2, 3, 0, 1],
                &[Moved, Moved, MovedIndirectly, MovedIndirectly],
            ),
            OP_DEPTH => o(&[], &[Integer], &[0], &[Added]),
            OP_DROP => o(&[T], &[], &[], &[]),
            OP_DUP => o(&[T], &[T, T], &[0, 0], &[Observed, Added]),
            OP_NIP => o(&[T, T], &[T], &[1], &[MovedIndirectly]),
            OP_OVER => o(
                &[T, T],
                &[T, T, T],
                &[0, 1, 0],
                &[Observed, Untouched, Added],
            ),
            OP_PICK => u(&[Integer], &[T], &[Added]),
            OP_ROLL => u(&[Integer], &[T], &[Moved]),
            OP_ROT => o(
                &[T, T, T],
                &[T, T, T],
                &[1, 2, 0],
                &[MovedIndirectly, MovedIndirectly, Moved],
            ),
            OP_SWAP => o(&[T, T], &[T, T], &[1, 0], &[MovedIndirectly, Moved]),
            OP_TUCK => o(
                &[T, T],
                &[T, T, T],
                &[1, 0, 1],
                &[Added, MovedIndirectly, MovedIndirectly],
            ),

            OP_CAT => u(
                &[ByteArray(None), ByteArray(None)],
                &[ByteArray(None)],
                &[Changed],
            ),
            OP_SPLIT => u(
                &[ByteArray(None), Integer],
                &[ByteArray(None), ByteArray(None)],
                &[Changed, Changed],
            ),
            OP_NUM2BIN => u(&[Integer, Integer], &[ByteArray(None)], &[Changed]),
            OP_BIN2NUM => u(&[ByteArray(None)], &[Integer], &[Changed]),
            OP_SIZE => u(
                &[ByteArray(None)],
                &[ByteArray(None), Integer],
                &[Observed, Added],
            ),

            OP_AND => u(
                &[ByteArray(None), ByteArray(None)],
                &[ByteArray(None)],
                &[Changed],
            ),
            OP_OR => u(
                &[ByteArray(None), ByteArray(None)],
                &[ByteArray(None)],
                &[Changed],
            ),
            OP_XOR => u(
                &[ByteArray(None), ByteArray(None)],
                &[ByteArray(None)],
                &[Changed],
            ),
            OP_EQUAL => u(&[T, T], &[Boolean], &[Changed]),
            OP_EQUALVERIFY => u(&[T, T], &[], &[]),

            OP_1ADD => u(&[Integer], &[Integer], &[Changed]),
            OP_1SUB => u(&[Integer], &[Integer], &[Changed]),
            OP_NEGATE => u(&[Integer], &[Integer], &[Changed]),
            OP_ABS => u(&[Integer], &[Integer], &[Changed]),
            OP_NOT => u(&[Boolean], &[Boolean], &[Changed]),
            OP_0NOTEQUAL => u(&[Integer], &[Boolean], &[Changed]),

            OP_ADD => u(&[Integer, Integer], &[Integer], &[Changed]),
            OP_SUB => u(&[Integer, Integer], &[Integer], &[Changed]),
            OP_DIV => u(&[Integer, Integer], &[Integer], &[Changed]),
            OP_MOD => u(&[Integer, Integer], &[Integer], &[Changed]),
            OP_BOOLAND => u(&[Boolean, Boolean], &[Boolean], &[Changed]),
            OP_BOOLOR => u(&[Boolean, Boolean], &[Boolean], &[Changed]),
            OP_NUMEQUAL => u(&[Integer, Integer], &[Boolean], &[Changed]),
            OP_NUMEQUALVERIFY => u(&[Integer, Integer], &[], &[]),
            OP_NUMNOTEQUAL => u(&[Integer, Integer], &[Boolean], &[Changed]),
            OP_LESSTHAN => u(&[Integer, Integer], &[Boolean], &[Changed]),
            OP_GREATERTHAN => u(&[Integer, Integer], &[Boolean], &[Changed]),
            OP_LESSTHANOREQUAL => u(&[Integer, Integer], &[Boolean], &[Changed]),
            OP_GREATERTHANOREQUAL => u(&[Integer, Integer], &[Boolean], &[Changed]),
            OP_MIN => u(&[Integer, Integer], &[Integer], &[Changed]),
            OP_MAX => u(&[Integer, Integer], &[Integer], &[Changed]),
            OP_WITHIN => u(&[Integer, Integer, Integer], &[Boolean], &[Changed]),

            OP_RIPEMD160 => u(&[ByteArray(None)], &[ByteArray(Some(20))], &[Changed]),
            OP_SHA1 => u(&[ByteArray(None)], &[ByteArray(Some(20))], &[Changed]),
            OP_SHA256 => u(&[ByteArray(None)], &[ByteArray(Some(32))], &[Changed]),
            OP_HASH160 => u(&[ByteArray(None)], &[ByteArray(Some(20))], &[Changed]),
            OP_HASH256 => u(&[ByteArray(None)], &[ByteArray(Some(32))], &[Changed]),
            OP_CHECKSIG => u(&[ByteArray(None), ByteArray(None)], &[Boolean], &[Changed]),
            OP_CHECKSIGVERIFY => u(&[ByteArray(None), ByteArray(None)], &[], &[Changed]),

            OP_CHECKDATASIG => u(
                &[ByteArray(None), ByteArray(None), ByteArray(None)],
                &[Boolean],
                &[Changed],
            ),
            OP_CHECKDATASIGVERIFY => u(
                &[ByteArray(None), ByteArray(None), ByteArray(None)],
                &[],
                &[Changed],
            ),
            OP_0 | OP_1NEGATE | OP_1 | OP_2 | OP_3 | OP_4 | OP_5 | OP_6 | OP_7 | OP_8 | OP_9
            | OP_10 | OP_11 | OP_12 | OP_13 | OP_14 | OP_15 | OP_16 => u(&[], &[Integer], &[Added]),

            OP_IF => u(&[T], &[], &[]),
            OP_ELSE => u(&[], &[], &[]),
            OP_ENDIF => u(&[], &[], &[]),

            OP_TOALTSTACK => u(&[T], &[], &[]),
            OP_FROMALTSTACK => u(&[], &[T], &[Added]),

            OP_IFDUP | OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {
                panic!("Opcode behavior cannot be expressed in OpcodeBehavior")
            }

            opcode if opcode.is_disabled() => panic!("Opcode is disabled"),

            _ => u(&[], &[], &[]),
        }
    }
}
