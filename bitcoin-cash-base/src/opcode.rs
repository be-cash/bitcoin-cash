use num_derive::*;

use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::data_type::{BitcoinBoolean, BitcoinByteArray, BitcoinInteger, DataType};
use crate::Integer;

lazy_static! {
    pub static ref MAP_NAME_TO_ENUM: HashMap<String, Opcode> = {
        let mut map = HashMap::new();
        map.insert("OP_0".to_string(), Opcode::OP_0);
        map.insert("OP_1NEGATE".to_string(), Opcode::OP_1NEGATE);
        for code in 0x51..Opcode::FIRST_UNDEFINED_OP_VALUE as u8 {
            let opcode: Opcode = num::FromPrimitive::from_u8(code)
                .unwrap_or_else(|| panic!("Invalid opcode {}", code));
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

/// All opcodes which can be used in a Bitcoin Cash script.
///
/// Can be used in `#[bitcoin_cash::script]` functions.
///
/// ## Example
/// ```
/// use bitcoin_cash::{Opcode::*, Address, ByteArray, Hashed};
/// struct Params {
///   address: Address<'static>,
/// }
/// #[bitcoin_cash::script(P2pkhInputs)]
/// fn p2pkh_script(params: Params, signature: ByteArray, public_key: ByteArray) {
///   OP_DUP(public_key);
///   let pkh = OP_HASH160(public_key);
///   let address = { params.address.hash().as_slice() };
///   OP_EQUALVERIFY(pkh, address);
///   OP_CHECKSIG(signature, public_key);
/// }
/// ```
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, FromPrimitive, IntoStaticStr)]
pub enum Opcode {
    /// ```text
    /// OP_0() -> Integer
    /// ```
    ///
    /// Pushes the integer 0 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let zero = OP_0;
    /// let expected = 0;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(zero, expected);
    /// # }
    /// ```
    OP_0 = 0x00,

    /// Pushes the next byte number of bytes. Not to be used in `#[bitcoin_cash::script]`
    /// functions.
    OP_PUSHDATA1 = 0x4c,

    /// Pushes the next two byte number of bytes. Not to be used in `#[bitcoin_cash::script]`
    /// functions.
    OP_PUSHDATA2 = 0x4d,

    /// Pushes the next four byte number of bytes. Not to be used in `#[bitcoin_cash::script]`
    /// functions.
    OP_PUSHDATA4 = 0x4e,

    /// ```text
    /// OP_1NEGATE() -> Integer
    /// ```
    ///
    /// Pushes the integer -1 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push -1
    /// let minus_one = OP_1NEGATE;
    ///
    /// let expected = -1;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(minus_one, expected);
    /// # }
    /// ```
    OP_1NEGATE = 0x4f,

    /// Reserved opcode. Fails script if in executed branch immediately.
    OP_RESERVED = 0x50,

    /// ```text
    /// OP_1() -> Integer
    /// ```
    ///
    /// Pushes the integer 1 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 1
    /// let one = OP_1;
    ///
    /// let expected = 1;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(one, expected);
    /// # }
    /// ```
    OP_1 = 0x51,

    /// ```text
    /// OP_2() -> Integer
    /// ```
    ///
    /// Pushes the integer 2 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 2
    /// let two = OP_2;
    ///
    /// let expected = 2;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(two, expected);
    /// # }
    /// ```
    OP_2 = 0x52,

    /// ```text
    /// OP_3() -> Integer
    /// ```
    ///
    /// Pushes the integer 3 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 3
    /// let three = OP_3;
    ///
    /// let expected = 3;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(three, expected);
    /// # }
    /// ```
    OP_3 = 0x53,

    /// ```text
    /// OP_4() -> Integer
    /// ```
    ///
    /// Pushes the integer 4 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 4
    /// let four = OP_4;
    ///
    /// let expected = 4;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(four, expected);
    /// # }
    /// ```
    OP_4 = 0x54,

    /// ```text
    /// OP_5() -> Integer
    /// ```
    ///
    /// Pushes the integer 5 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 5
    /// let five = OP_5;
    ///
    /// let expected = 5;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(five, expected);
    /// # }
    /// ```
    OP_5 = 0x55,

    /// ```text
    /// OP_6() -> Integer
    /// ```
    ///
    /// Pushes the integer 6 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 6
    /// let six = OP_6;
    ///
    /// let expected = 6;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(six, expected);
    /// # }
    /// ```
    OP_6 = 0x56,

    /// ```text
    /// OP_7() -> Integer
    /// ```
    ///
    /// Pushes the integer 7 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 7
    /// let seven = OP_7;
    ///
    /// let expected = 7;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(seven, expected);
    /// # }
    /// ```
    OP_7 = 0x57,

    /// ```text
    /// OP_8() -> Integer
    /// ```
    ///
    /// Pushes the integer 8 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 8
    /// let eight = OP_8;
    ///
    /// let expected = 8;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(eight, expected);
    /// # }
    /// ```
    OP_8 = 0x58,

    /// ```text
    /// OP_9() -> Integer
    /// ```
    ///
    /// Pushes the integer 9 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 9
    /// let nine = OP_9;
    ///
    /// let expected = 9;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(nine, expected);
    /// # }
    /// ```
    OP_9 = 0x59,

    /// ```text
    /// OP_10() -> Integer
    /// ```
    ///
    /// Pushes the integer 10 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 10
    /// let ten = OP_10;
    ///
    /// let expected = 10;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(ten, expected);
    /// # }
    /// ```
    OP_10 = 0x5a,

    /// ```text
    /// OP_11() -> Integer
    /// ```
    ///
    /// Pushes the integer 11 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 11
    /// let eleven = OP_11;
    ///
    /// let expected = 11;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(eleven, expected);
    /// # }
    /// ```
    OP_11 = 0x5b,

    /// ```text
    /// OP_12() -> Integer
    /// ```
    ///
    /// Pushes the integer 12 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 12
    /// let twelve = OP_12;
    ///
    /// let expected = 12;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(twelve, expected);
    /// # }
    /// ```
    OP_12 = 0x5c,

    /// ```text
    /// OP_13() -> Integer
    /// ```
    ///
    /// Pushes the integer 13 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 13
    /// let thirteen = OP_13;
    ///
    /// let expected = 13;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(thirteen, expected);
    /// # }
    /// ```
    OP_13 = 0x5d,

    /// ```text
    /// OP_14() -> Integer
    /// ```
    ///
    /// Pushes the integer 14 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 14
    /// let fourteen = OP_14;
    ///
    /// let expected = 14;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(fourteen, expected);
    /// # }
    /// ```
    OP_14 = 0x5e,

    /// ```text
    /// OP_15() -> Integer
    /// ```
    ///
    /// Pushes the integer 15 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 15
    /// let fiveteen = OP_15;
    ///
    /// let expected = 15;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(fiveteen, expected);
    /// # }
    /// ```
    OP_15 = 0x5f,

    /// ```text
    /// OP_16() -> Integer
    /// ```
    ///
    /// Pushes the integer 16 onto the stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // push 16
    /// let sixteen = OP_16;
    ///
    /// let expected = 16;  // equivalent (and prefered) way to push numbers
    /// OP_EQUALVERIFY(sixteen, expected);
    /// # }
    /// ```
    OP_16 = 0x60,

    /// ```text
    /// OP_NOP() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP();
    /// # }
    /// ```
    OP_NOP = 0x61,

    /// Reserved opcode. Fails script if in executed branch immediately.
    OP_VER = 0x62,

    /// ```text
    /// OP_IF(condition: bool) -> ()
    /// ```
    ///
    /// Creates a branch (`OP_IF`/`OP_ELSE`/`OP_ENDIF` construct) that will only execute
    /// if `condition` is true and will execute the alternative branch otherwise.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let condition = true;
    ///
    /// // branch on condition
    /// OP_IF(condition); {
    ///     let result = 5;
    /// } OP_ELSE; {
    ///     let result = 6;
    /// } OP_ENDIF;
    ///
    /// let expected = 5;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_IF = 0x63,

    /// Opposite of OP_NOTIF. Currently not supported in `#[bitcoin_cash::script]` functions.
    OP_NOTIF = 0x64,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_VERIF = 0x65,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_VERNOTIF = 0x66,

    /// ```text
    /// OP_ELSE() -> ()
    /// ```
    ///
    /// Demarcates a branch (`OP_IF`/`OP_ELSE`/`OP_ENDIF` construct) that will only execute
    /// if `condition` is true and will execute the alternative branch otherwise.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let condition = true;
    ///
    /// // branch on condition
    /// OP_IF(condition); {
    ///     let result = 5;
    /// } OP_ELSE; {
    ///     let result = 6;
    /// } OP_ENDIF;
    ///
    /// let expected = 5;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_ELSE = 0x67,

    /// ```text
    /// OP_ENDIF() -> ()
    /// ```
    ///
    /// Ends a branch (`OP_IF`/`OP_ELSE`/`OP_ENDIF` construct) that will only execute
    /// if `condition` is true and will execute the alternative branch otherwise.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let condition = true;
    ///
    /// // branch on condition
    /// OP_IF(condition); {
    ///     let result = 5;
    /// } OP_ELSE; {
    ///     let result = 6;
    /// } OP_ENDIF;
    ///
    /// let expected = 5;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_ENDIF = 0x68,

    /// ```text
    /// OP_VERIFY(condition: bool) -> ()
    /// ```
    ///
    /// Marks transaction as invalid if `condition` is not true.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let condition = true;
    ///
    /// // verify condition
    /// OP_VERIFY(condition);
    /// # }
    /// ```
    OP_VERIFY = 0x69,

    /// Fails execution of the script immediately. Used to attach data to transactions.
    OP_RETURN = 0x6a,

    /// ```text
    /// OP_TOALTSTACK<T>(item: T) -> ()
    /// ```
    ///
    /// Moves `item` to the top of the altstack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let item = b"Bitcoin Cash";
    ///
    /// // move item to altstack
    /// OP_TOALTSTACK(item);
    ///
    /// let expected = b"Bitcoin Cash";
    /// OP_FROMALTSTACK(item);  // Note: retains name "item" on altstack
    /// OP_EQUALVERIFY(expected, item);
    /// # }
    /// ```
    OP_TOALTSTACK = 0x6b,

    /// ```text
    /// OP_FROMALTSTACK<T>(altitem: T) -> T
    /// ```
    ///
    /// Moves the top item of the altstack to the main stack.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let item = b"Bitcoin Cash";
    /// OP_TOALTSTACK(item);
    /// let expected = b"Bitcoin Cash";
    ///
    /// // move item back to main stack
    /// OP_FROMALTSTACK(item);  // Note: retains name "item" on main stack
    ///
    /// OP_EQUALVERIFY(expected, item);
    /// # }
    /// ```
    OP_FROMALTSTACK = 0x6c,

    /// ```text
    /// OP_2DROP<T, U>(a: T, b: U) -> ()
    /// ```
    ///
    /// Drops the two top stack items.
    ///
    /// ```text
    /// a b ->
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    ///
    /// // drop b and c
    /// OP_2DROP(b, c);
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_2DROP = 0x6d,

    /// ```text
    /// OP_2DUP<T, U>(a: T, b: U) -> (T, U, T, U)
    /// ```
    ///
    /// Duplicates the top two stack items.
    ///
    /// ```text
    /// a b -> a b a b
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // duplicate a and b
    /// OP_2DUP(a, b);  // Note: duplicated items retain name
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_2DUP = 0x6e,

    /// ```text
    /// OP_3DUP<T, U, V>(a: T, b: U, c: V) -> (T, U, V, T, U, V)
    /// ```
    ///
    /// Duplicates the top two three items.
    ///
    /// ```text
    /// a b c -> a b c a b c
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    ///
    /// // duplicate a, b and c
    /// OP_3DUP(a, b, c);  // Note: duplicated items retain name
    ///
    /// let expected = b"C";
    /// OP_EQUALVERIFY(c, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"C";
    /// OP_EQUALVERIFY(c, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_3DUP = 0x6f,

    /// ```text
    /// OP_2OVER<T, U, V, W>(a: T, b: U, c: V, d: W) -> (T, U, V, W, T, U)
    /// ```
    ///
    /// Copies the pair of items two spaces back in the stack to the front.
    ///
    /// ```text
    /// a b _ _ -> a b _ _ a b
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    /// let d = b"D";
    ///
    /// // duplicate a and b
    /// OP_2OVER(a, b, __, __);  // Note: duplicated items retain name
    ///
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"D";
    /// OP_EQUALVERIFY(d, expected);
    /// let expected = b"C";
    /// OP_EQUALVERIFY(c, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_2OVER = 0x70,

    /// ```text
    /// OP_2ROT<T, U, V, W, X, Y>(a: T, b: U, c: V, d: W, e: X, f: Y) -> (V, W, X, Y, T, U)
    /// ```
    ///
    /// The fifth and sixth items back are moved to the top of the stack.
    ///
    /// ```text
    /// a b _ _ _ _ -> _ _ _ _ a b
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    /// let d = b"D";
    /// let e = b"E";
    /// let f = b"F";
    ///
    /// // move a and b to the top
    /// OP_2ROT(a, b, __, __, __, __);  // Note: moved items retain name
    ///
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"F";
    /// OP_EQUALVERIFY(f, expected);
    /// let expected = b"E";
    /// OP_EQUALVERIFY(e, expected);
    /// let expected = b"D";
    /// OP_EQUALVERIFY(d, expected);
    /// let expected = b"C";
    /// OP_EQUALVERIFY(c, expected);
    /// # }
    /// ```
    OP_2ROT = 0x71,

    /// ```text
    /// OP_2SWAP<T, U, V, W>(a: T, b: U, c: V, d: W) -> (V, W, T, U)
    /// ```
    ///
    /// Swaps the top two pairs of items.
    ///
    /// ```text
    /// a b c d -> c d a b
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    /// let d = b"D";
    ///
    /// // swap a and b with c and d
    /// OP_2SWAP(a, b, c, d);  // Note: moved items retain name
    ///
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"D";
    /// OP_EQUALVERIFY(d, expected);
    /// let expected = b"C";
    /// OP_EQUALVERIFY(c, expected);
    /// # }
    /// ```
    OP_2SWAP = 0x72,

    /// If the top stack value is true, duplicate it. Not to be used in `#[bitcoin_cash::script]`
    /// functions.
    OP_IFDUP = 0x73,

    /// ```text
    /// OP_DEPTH() -> Integer
    /// ```
    ///
    /// Returns the number of stack items.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    ///
    /// // number of items on the stack
    /// let n = OP_DEPTH();
    ///
    /// let expected = 3;
    /// OP_EQUALVERIFY(n, expected);
    /// # }
    /// ```
    OP_DEPTH = 0x74,

    /// ```text
    /// OP_DROP<T>(a: T) -> ()
    /// ```
    ///
    /// Drops the top stack item.
    ///
    /// ```text
    /// a ->
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // drop b
    /// OP_DROP(b);
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_DROP = 0x75,

    /// ```text
    /// OP_DUP<T>(a: T) -> (T, T)
    /// ```
    ///
    /// Duplicates the top stack item.
    ///
    /// ```text
    /// a -> a a
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    ///
    /// // drop b and c
    /// OP_DUP(a);  // Note: duplicated item retains name
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_DUP = 0x76,

    /// ```text
    /// OP_NIP<T, U>(a: T, b: U) -> T
    /// ```
    ///
    /// Removes the second-to-top stack item.
    ///
    /// ```text
    /// a b -> b
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // drop a
    /// OP_NIP(a, __);
    ///
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// # }
    /// ```
    OP_NIP = 0x77,

    /// ```text
    /// OP_OVER<T, U>(a: T, b: U) -> (T, U, T)
    /// ```
    ///
    /// Copies the second-to-top stack item to the top of the stack.
    ///
    /// ```text
    /// a b -> a b a
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // copy a
    /// OP_OVER(a, __);  // Note: duplicated item retains name
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_OVER = 0x78,

    /// ```text
    /// OP_PICK<T>(n: Integer) -> T
    /// ```
    ///
    /// The item `n` back in the stack is copied to the top.
    ///
    /// ```text
    /// xₙ ... x₂ x₁ x₀ <n> -> xₙ ... x₂ x₁ x₀ xₙ
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // calculate depth of a
    /// let depth_a = depth_of(a);
    /// // copy a
    /// OP_PICK(depth_a);  // Note: duplicated item retains name
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_PICK = 0x79,

    /// ```text
    /// OP_ROLL<T>(n: Integer) -> T
    /// ```
    ///
    /// The item `n` back in the stack is moved to the top.
    ///
    /// ```text
    /// xₙ ... x₂ x₁ x₀ <n> -> xₙ₋₁ ... x₂ x₁ x₀ xₙ
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // calculate depth of a
    /// let depth_a = depth_of(a);
    /// // move a to the top of the stack
    /// OP_ROLL(depth_a);  // Note: moved item retains name
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// # }
    /// ```
    OP_ROLL = 0x7a,

    /// ```text
    /// OP_ROT<T, U, V>(a: T, b: U, c: V) -> (U, V, T)
    /// ```
    ///
    /// The third-to-top item is moved to the top.
    ///
    /// ```text
    /// a b c -> b c a
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    /// let c = b"C";
    ///
    /// // move a to the top
    /// OP_ROT(a, __, __);
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"C";
    /// OP_EQUALVERIFY(c, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// # }
    /// ```
    OP_ROT = 0x7b,

    /// ```text
    /// OP_SWAP<T, U>(a: T, b: U) -> (U, T)
    /// ```
    ///
    /// The top two items on the stack are swapped.
    ///
    /// ```text
    /// a b -> b a
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // swap a and b
    /// OP_SWAP(a, b);
    ///
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// # }
    /// ```
    OP_SWAP = 0x7c,

    /// ```text
    /// OP_TUCK<T, U>(a: T, b: U) -> (U, T, U)
    /// ```
    ///
    /// The item at the top of the stack is copied and inserted before the second-to-top item.
    ///
    /// ```text
    /// a b -> b a b
    /// ```
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"A";
    /// let b = b"B";
    ///
    /// // insert b before a
    /// OP_TUCK(__, b);
    ///
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"A";
    /// OP_EQUALVERIFY(a, expected);
    /// let expected = b"B";
    /// OP_EQUALVERIFY(b, expected);
    /// # }
    /// ```
    OP_TUCK = 0x7d,

    /// ```text
    /// OP_CAT(left: ByteArray, right: ByteArray) -> ByteArray
    /// ```
    ///
    /// Concatenates two byte sequences.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"Bitcoin";
    /// let b = b"Cash";
    ///
    /// // concatenate a and b
    /// let ab = OP_CAT(a, b);
    ///
    /// let expected = b"BitcoinCash";
    /// OP_EQUALVERIFY(ab, expected);
    /// # }
    /// ```
    OP_CAT = 0x7e,

    /// ```text
    /// OP_SPLIT(array: ByteArray, split_index: Integer) -> (ByteArray, ByteArray)
    /// ```
    ///
    /// Split `array` at `split_index`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    /// let split_index = 7;
    ///
    /// // split array at index 7
    /// let (a, b) = OP_SPLIT(array, split_index);
    ///
    /// let expected = b"Cash";
    /// OP_EQUALVERIFY(b, expected);
    /// let expected = b"Bitcoin";
    /// OP_EQUALVERIFY(a, expected);
    /// # }
    /// ```
    OP_SPLIT = 0x7f,

    /// ```text
    /// OP_NUM2BIN(num: Integer, n_bytes: Integer) -> ByteArray
    /// ```
    ///
    /// Convert `num`into a byte sequence of size `n_bytes`, taking account of
    /// the sign bit. The byte sequence produced uses little-endian encoding.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let num = 0x1337;
    /// let n_bytes = 2;
    ///
    /// // encode num as 2 bytes
    /// let num2le = OP_NUM2BIN(num, n_bytes);
    ///
    /// let expected = b"\x37\x13";
    /// OP_EQUALVERIFY(num2le, expected);
    /// # }
    /// ```
    OP_NUM2BIN = 0x80,

    /// ```text
    /// OP_BIN2NUM(array: ByteArray) -> Integer
    /// ```
    ///
    /// Convert `array` into a numeric value, including minimal encoding.
    /// `array` must encode the value in little-endian encoding.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let num_encoded = b"\x37\x13";
    ///
    /// // decode num_encoded
    /// let num = OP_BIN2NUM(num_encoded);
    ///
    /// let expected = 0x1337;
    /// OP_EQUALVERIFY(num, expected);
    /// # }
    /// ```
    OP_BIN2NUM = 0x81,

    /// ```text
    /// OP_SIZE(array: ByteArray) -> (ByteArray, Integer)
    /// ```
    ///
    /// Returns the byte length of `array`, retaining `array`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // calculate size of array
    /// let (__, size) = OP_SIZE(array);
    ///
    /// let expected = 11;
    /// OP_EQUALVERIFY(size, expected);
    /// let expected = b"BitcoinCash";
    /// OP_EQUALVERIFY(array, expected);
    /// # }
    /// ```
    OP_SIZE = 0x82,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_INVERT = 0x83,

    /// ```text
    /// OP_AND(a: ByteArray, b: ByteArray) -> ByteArray
    /// ```
    ///
    /// Boolean AND between each bit of `a` and `b`.
    ///
    /// The two operands must be the same size.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"\x0103";
    /// let b = b"\x0302";
    ///
    /// // calculate a & b
    /// let bits = OP_AND(a, b);
    ///
    /// let expected = b"\x0102";
    /// OP_EQUALVERIFY(bits, expected);
    /// # }
    /// ```
    OP_AND = 0x84,

    /// ```text
    /// OP_OR(a: ByteArray, b: ByteArray) -> ByteArray
    /// ```
    ///
    /// Boolean OR between each bit of `a` and `b`.
    ///
    /// The two operands must be the same size.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"\x0103";
    /// let b = b"\x0201";
    ///
    /// // calculate a | b
    /// let bits = OP_OR(a, b);
    ///
    /// let expected = b"\x0203";
    /// OP_EQUALVERIFY(bits, expected);
    /// # }
    /// ```
    OP_OR = 0x85,

    /// ```text
    /// OP_XOR(a: ByteArray, b: ByteArray) -> ByteArray
    /// ```
    ///
    /// Boolean XOR between each bit of `a` and `b`.
    ///
    /// The two operands must be the same size.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"\x0103";
    /// let b = b"\x0302";
    ///
    /// // calculate a ^ b
    /// let bits = OP_XOR(a, b);
    ///
    /// let expected = b"\x0201";
    /// OP_EQUALVERIFY(bits, expected);
    /// # }
    /// ```
    OP_XOR = 0x86,

    /// ```text
    /// OP_EQUAL<T>(a: T, b: T) -> bool
    /// ```
    ///
    /// Returns whether `a == b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"BitcoinCash";
    /// let b = b"BitcoinCash";
    ///
    /// // check if a == b
    /// let is_equal = OP_EQUAL(a, b);
    ///
    /// OP_VERIFY(is_equal);
    /// # }
    /// ```
    OP_EQUAL = 0x87,

    /// ```text
    /// OP_EQUALVERIFY<T>(a: T, b: T) -> ()
    /// ```
    ///
    /// Verifies that `a == b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = b"BitcoinCash";
    /// let b = b"BitcoinCash";
    ///
    /// // verify that a == b
    /// OP_EQUALVERIFY(a, b);
    /// # }
    /// ```
    OP_EQUALVERIFY = 0x88,

    /// Reserved opcode. Fails script if in executed branch immediately.
    OP_RESERVED1 = 0x89,

    /// Reserved opcode. Fails script if in executed branch immediately.
    OP_RESERVED2 = 0x8a,

    /// ```text
    /// OP_1ADD(a: Integer) -> Integer
    /// ```
    ///
    /// Calculates `a + 1`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    ///
    /// // calculate a + 1
    /// let result = OP_1ADD(a);
    ///
    /// let expected = 8;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_1ADD = 0x8b,

    /// ```text
    /// OP_1SUB(a: Integer) -> Integer
    /// ```
    ///
    /// Calculates `a - 1`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    ///
    /// // calculate a - 1
    /// let result = OP_1SUB(a);
    ///
    /// let expected = 6;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_1SUB = 0x8c,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_2MUL = 0x8d,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_2DIV = 0x8e,

    /// ```text
    /// OP_NEGATE(a: Integer) -> Integer
    /// ```
    ///
    /// Calculates `-a`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    ///
    /// // calculate -a
    /// let result = OP_NEGATE(a);
    ///
    /// let expected = -7;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_NEGATE = 0x8f,

    /// ```text
    /// OP_ABS(a: Integer) -> Integer
    /// ```
    ///
    /// Calculates `abs(a)`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = -7;
    ///
    /// // calculate abs(a)
    /// let result = OP_ABS(a);
    ///
    /// let expected = 7;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_ABS = 0x90,

    /// ```text
    /// OP_NOT(a: Integer) -> Integer
    /// ```
    ///
    /// Calculates `!a`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = false;
    ///
    /// // calculate !a
    /// let result = OP_NOT(a);
    ///
    /// OP_VERIFY(result);
    /// # }
    /// ```
    OP_NOT = 0x91,

    /// ```text
    /// OP_0NOTEQUAL(a: Integer) -> bool
    /// ```
    ///
    /// Calculates `a != 0`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    ///
    /// // calculate a != 0
    /// let result = OP_0NOTEQUAL(a);
    ///
    /// OP_VERIFY(result);
    /// # }
    /// ```
    OP_0NOTEQUAL = 0x92,

    /// ```text
    /// OP_ADD(a: Integer, b: Integer) -> Integer
    /// ```
    ///
    /// Calculates `a + b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 3;
    ///
    /// // calculate a + b
    /// let result = OP_ADD(a, b);
    ///
    /// let expected = 10;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_ADD = 0x93,

    /// ```text
    /// OP_SUB(a: Integer, b: Integer) -> Integer
    /// ```
    ///
    /// Calculates `a - b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 3;
    ///
    /// // calculate a - b
    /// let result = OP_SUB(a, b);
    ///
    /// let expected = 4;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_SUB = 0x94,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_MUL = 0x95,

    /// ```text
    /// OP_DIV(a: Integer, b: Integer) -> Integer
    /// ```
    ///
    /// Calculates `a / b`, truncating the fraction part.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 3;
    ///
    /// // calculate a / b
    /// let result = OP_DIV(a, b);
    ///
    /// let expected = 2;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_DIV = 0x96,

    /// ```text
    /// OP_MOD(a: Integer, b: Integer) -> Integer
    /// ```
    ///
    /// Calculates `a % b` (a modulo b).
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 3;
    ///
    /// // calculate a % b
    /// let result = OP_MOD(a, b);
    ///
    /// let expected = 1;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_MOD = 0x97,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_LSHIFT = 0x98,

    /// Disabled opcode. Fails script immediately even if not in executed branch.
    OP_RSHIFT = 0x99,

    /// ```text
    /// OP_BOOLAND(a: bool, b: bool) -> bool
    /// ```
    ///
    /// Calculates `a && b`, boolean AND (∧).
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = false;
    /// let b = true;
    ///
    /// // calculate a && b
    /// let result = OP_BOOLAND(a, b);
    ///
    /// let expected = false;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_BOOLAND = 0x9a,

    /// ```text
    /// OP_BOOLOR(a: bool, b: bool) -> bool
    /// ```
    ///
    /// Calculates `a || b`, boolean OR (∨).
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = false;
    /// let b = true;
    ///
    /// // calculate a || b
    /// let result = OP_BOOLOR(a, b);
    ///
    /// OP_VERIFY(result);
    /// # }
    /// ```
    OP_BOOLOR = 0x9b,

    /// ```text
    /// OP_NUMEQUAL(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `a == b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 7;
    ///
    /// // check if a == b
    /// let is_equal = OP_NUMEQUAL(a, b);
    ///
    /// OP_VERIFY(is_equal);
    /// # }
    /// ```
    OP_NUMEQUAL = 0x9c,

    /// ```text
    /// OP_NUMEQUALVERIFY(a: Integer, b: Integer) -> ()
    /// ```
    ///
    /// Verifies that `a == b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 7;
    ///
    /// // verify that a == b
    /// OP_NUMEQUALVERIFY(a, b);
    /// # }
    /// ```
    OP_NUMEQUALVERIFY = 0x9d,

    /// ```text
    /// OP_NUMNOTEQUAL(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `a != b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 9;
    ///
    /// // check if a == b
    /// let is_not_equal = OP_NUMNOTEQUAL(a, b);
    ///
    /// OP_VERIFY(is_not_equal);
    /// # }
    /// ```
    OP_NUMNOTEQUAL = 0x9e,

    /// ```text
    /// OP_LESSTHAN(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `a < b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 9;
    ///
    /// // check if a < b
    /// let is_less = OP_LESSTHAN(a, b);
    ///
    /// OP_VERIFY(is_less);
    /// # }
    /// ```
    OP_LESSTHAN = 0x9f,

    /// ```text
    /// OP_GREATERTHAN(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `a > b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 11;
    /// let b = 9;
    ///
    /// // check if a > b
    /// let is_greater = OP_GREATERTHAN(a, b);
    ///
    /// OP_VERIFY(is_greater);
    /// # }
    /// ```
    OP_GREATERTHAN = 0xa0,

    /// ```text
    /// OP_LESSTHANOREQUAL(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `a <= b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 7;
    /// let b = 7;
    ///
    /// // check if a <= b
    /// let is_at_most = OP_LESSTHANOREQUAL(a, b);
    ///
    /// OP_VERIFY(is_at_most);
    /// # }
    /// ```
    OP_LESSTHANOREQUAL = 0xa1,

    /// ```text
    /// OP_GREATERTHANOREQUAL(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `a >= b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 9;
    /// let b = 9;
    ///
    /// // check if a >= b
    /// let is_at_least = OP_GREATERTHANOREQUAL(a, b);
    ///
    /// OP_VERIFY(is_at_least);
    /// # }
    /// ```
    OP_GREATERTHANOREQUAL = 0xa2,

    /// ```text
    /// OP_MIN(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `min(a, b)`, the lesser of `a` and `b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 3;
    /// let b = 9;
    ///
    /// // calculate min(a, b)
    /// let result = OP_MIN(a, b);
    ///
    /// let expected = 3;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_MIN = 0xa3,

    /// ```text
    /// OP_MAX(a: Integer, b: Integer) -> bool
    /// ```
    ///
    /// Calculates `max(a, b)`, the greater of `a` and `b`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 3;
    /// let b = 9;
    ///
    /// // calculate max(a, b)
    /// let result = OP_MAX(a, b);
    ///
    /// let expected = 9;
    /// OP_EQUALVERIFY(result, expected);
    /// # }
    /// ```
    OP_MAX = 0xa4,

    /// ```text
    /// OP_WITHIN(a: Integer, min: Integer, max: Integer) -> bool
    /// ```
    ///
    /// Calculates `a >= min && a <= max`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let a = 3;
    /// let min = 1;
    /// let max = 9;
    ///
    /// // calculate if a is within min and max
    /// let result = OP_WITHIN(a, min, max);
    ///
    /// OP_VERIFY(result);
    /// # }
    /// ```
    OP_WITHIN = 0xa5,

    /// ```text
    /// OP_RIPEMD160(array: ByteArray) -> ByteArray
    /// ```
    ///
    /// Hashes `array` using RIPEMD-160.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // calculate ripemd160(array)
    /// let hash = OP_RIPEMD160(array);
    ///
    /// let expected = hex!("0d2aa57463e5fac82f97f496ed98525fbec71c4c");
    /// OP_EQUALVERIFY(hash, expected);
    /// # }
    /// ```
    OP_RIPEMD160 = 0xa6,

    /// ```text
    /// OP_SHA1(array: ByteArray) -> ByteArray
    /// ```
    ///
    /// Hashes `array` using SHA-1.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // calculates sha1(array)
    /// let hash = OP_SHA1(array);
    ///
    /// let expected = hex!("a7a1986ab925f4d8a81fc0da1352c780ad2f5fe1");
    /// OP_EQUALVERIFY(hash, expected);
    /// # }
    /// ```
    OP_SHA1 = 0xa7,

    /// ```text
    /// OP_SHA256(array: ByteArray) -> ByteArray
    /// ```
    ///
    /// Hashes `array` using SHA-256.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // calculates sha256(array)
    /// let hash = OP_SHA256(array);
    ///
    /// let expected = hex!("78e015aa460c0a5be71fe4618c72898200a45a20f9bd7048398971babc3b372b");
    /// OP_EQUALVERIFY(hash, expected);
    /// # }
    /// ```
    OP_SHA256 = 0xa8,

    /// ```text
    /// OP_HASH160(array: ByteArray) -> ByteArray
    /// ```
    ///
    /// Hashes `array` using first SHA-256 and then RIPEMD-160.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // calculates ripemd160(sha256(array))
    /// let hash = OP_HASH160(array);
    ///
    /// let expected = hex!("29e99ecb43b5a4c19aa2b05c7d6fc439bca5f023");
    /// OP_EQUALVERIFY(hash, expected);
    /// # }
    /// ```
    OP_HASH160 = 0xa9,

    /// ```text
    /// OP_HASH256(array: ByteArray) -> ByteArray
    /// ```
    ///
    /// Hashes `array` twice using SHA-256.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // calculates sha256(sha256(array))
    /// let hash = OP_HASH256(array);
    ///
    /// let expected = hex!("575d8ad02159b76cf2beda18c4ccb9bdb9a7ac894506d97e319c9e5c3096ca37");
    /// OP_EQUALVERIFY(hash, expected);
    /// # }
    /// ```
    OP_HASH256 = 0xaa,

    /// ```text
    /// OP_CODESEPARATOR() -> ()
    /// ```
    ///
    /// Makes `OP_CHECK(MULTI)SIG(VERIFY)` set `scriptCode` to everything after
    /// the most recently-executed `OP_CODESEPARATOR` when computing the sighash.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // removes "BitcoinCash" from scriptCode
    /// OP_CODESEPARATOR();
    /// # }
    /// ```
    OP_CODESEPARATOR = 0xab,

    /// ```text
    /// OP_CHECKSIG(public_key: ByteArray, signature: ByteArray) -> bool
    /// ```
    ///
    /// The last byte (=sighash type) of `signature` is removed.
    /// The sighash for this input is calculated based on the sighash type.
    /// The truncated signature used by `OP_CHECKSIG` must be a valid ECDSA or
    /// Schnorr signature for that hash and `public_key`. If it is valid, 1 is
    /// returned, if it is empty, 0 is returned, otherwise the operation fails.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::{Opcode::*, ByteArray};
    /// # struct Params;
    /// #[bitcoin_cash::script(P2PKInputs)]
    /// fn p2pk(_: Params, signature: ByteArray) {
    ///   let public_key = hex!("0201961ef44067e870a9b1684041929caffad57eae6bbc79dd785320d53231f519");
    ///
    ///   // check if `signature` signs current sighash for `public_key`
    ///   let success = OP_CHECKSIG(signature, public_key);
    /// }
    /// ```
    OP_CHECKSIG = 0xac,

    /// ```text
    /// OP_CHECKSIGVERIFY(public_key: ByteArray, signature: ByteArray) -> ()
    /// ```
    ///
    /// The last byte (=sighash type) of `signature` is removed.
    /// The sighash for this input is calculated based on the sighash type.
    /// Verifies the truncated signature used by `OP_CHECKSIGVERIFY` is a
    /// valid ECDSA or Schnorr signature for that hash and `public_key`.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::{Opcode::*, ByteArray};
    /// # struct Params;
    /// #[bitcoin_cash::script(P2PKInputs)]
    /// fn p2pk(_: Params, signature: ByteArray) {
    ///   let public_key = hex!("0201961ef44067e870a9b1684041929caffad57eae6bbc79dd785320d53231f519");
    ///
    ///   // verify `signature` signs current sighash for `public_key`
    ///   OP_CHECKSIGVERIFY(signature, public_key);
    /// }
    /// ```
    OP_CHECKSIGVERIFY = 0xad,

    /// Performs a multisig check. Not to be used in `#[bitcoin_cash::script]` functions.
    OP_CHECKMULTISIG = 0xae,

    /// Verifies a multisig check. Not to be used in `#[bitcoin_cash::script]` functions.
    OP_CHECKMULTISIGVERIFY = 0xaf,

    /// ```text
    /// OP_NOP1() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP1();
    /// # }
    /// ```
    OP_NOP1 = 0xb0,

    /// ```text
    /// OP_CHECKLOCKTIMEVERIFY(locktime: Integer) -> Integer
    /// ```
    ///
    /// Marks transaction as invalid if the top stack item is greater than the transaction's
    /// `nLockTime` field, otherwise script evaluation continues as though an `OP_NOP` was
    /// executed. Transaction is also invalid if
    ///
    /// 1. the stack is empty or
    /// 2. the top stack item is negative or
    /// 3. the top stack item is greater than or equal to 500000000 while the transaction's
    ///    `nLockTime` field is less than 500000000, or vice versa or
    /// 4. the input's nSequence field is equal to 0xffffffff.
    ///
    /// The precise semantics are described in BIP65.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let locktime = 400_000;
    ///
    /// // verify locktime
    /// OP_CHECKLOCKTIMEVERIFY(locktime);
    ///
    /// let expected = 400_000;
    /// OP_EQUALVERIFY(locktime, expected);
    /// # }
    /// ```
    OP_CHECKLOCKTIMEVERIFY = 0xb1,

    /// ```text
    /// OP_CHECKSEQUENCEVERIFY(sequence: Integer) -> Integer
    /// ```
    ///
    /// Marks transaction as invalid if the relative lock time of the input
    /// (enforced by BIP68 with `nSequence`) is not equal to or longer than
    /// the value of the top stack item. The precise semantics are described
    /// in BIP112.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let sequence = 1000;
    ///
    /// // verify input age
    /// OP_CHECKSEQUENCEVERIFY(sequence);
    ///
    /// let expected = 1000;
    /// OP_EQUALVERIFY(sequence, expected);
    /// # }
    /// ```
    OP_CHECKSEQUENCEVERIFY = 0xb2,

    /// ```text
    /// OP_NOP4() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP4();
    /// # }
    /// ```
    OP_NOP4 = 0xb3,

    /// ```text
    /// OP_NOP5() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP5();
    /// # }
    /// ```
    OP_NOP5 = 0xb4,

    /// ```text
    /// OP_NOP6() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP6();
    /// # }
    /// ```
    OP_NOP6 = 0xb5,

    /// ```text
    /// OP_NOP7() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP7();
    /// # }
    /// ```
    OP_NOP7 = 0xb6,

    /// ```text
    /// OP_NOP8() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP8();
    /// # }
    /// ```
    OP_NOP8 = 0xb7,

    /// ```text
    /// OP_NOP9() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP9();
    /// # }
    /// ```
    OP_NOP9 = 0xb8,

    /// ```text
    /// OP_NOP10() -> ()
    /// ```
    ///
    /// Does nothing.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// // do nothing
    /// OP_NOP10();
    /// # }
    /// ```
    OP_NOP10 = 0xb9,

    /// ```text
    /// OP_CHECKDATASIG(signature: ByteArray, message: ByteArray, public_key: ByteArray) -> bool
    /// ```
    ///
    /// Checks whether `signature` is a valid ECDSA or Schnorr signature for `sha256(message)` and
    /// `public_key`.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let signature = [
    ///     hex!("3045022100f560a6e928ec52e77801a3ea4cbfbe6d89d1fff77a8d2ed00c457af278ae54").as_ref(),
    ///     hex!("01022015cc2b1c92b53cef6afd10e5e5fa33eb66b9d13d82d970a4a951b6e7f1903509").as_ref(),
    /// ].concat();
    /// let message = b"BitcoinCash";
    /// let public_key = hex!("0350be375c6e807988a5f575c9776e271e19a79f64681bcfe6a1affde9a5444496");
    ///
    /// let is_valid_sig = OP_CHECKDATASIG(signature, message, public_key);
    /// # }
    /// ```
    OP_CHECKDATASIG = 0xba,

    /// ```text
    /// OP_CHECKDATASIGVERIFY(signature: ByteArray, message: ByteArray, public_key: ByteArray) -> ()
    /// ```
    ///
    /// Verifies that `signature` is a valid ECDSA or Schnorr signature for `sha256(message)` and
    /// `public_key`.
    ///
    /// Usage:
    /// ```
    /// # use hex_literal::hex;
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let signature = [
    ///     hex!("3045022100f560a6e928ec52e77801a3ea4cbfbe6d89d1fff77a8d2ed00c457af278ae54").as_ref(),
    ///     hex!("01022015cc2b1c92b53cef6afd10e5e5fa33eb66b9d13d82d970a4a951b6e7f1903509").as_ref(),
    /// ].concat();
    /// let message = b"BitcoinCash";
    /// let public_key = hex!("0350be375c6e807988a5f575c9776e271e19a79f64681bcfe6a1affde9a5444496");
    ///
    /// OP_CHECKDATASIGVERIFY(signature, message, public_key);
    /// # }
    /// ```
    OP_CHECKDATASIGVERIFY = 0xbb,

    /// ```text
    /// OP_REVERSEBYTES(array: ByteArray) -> ByteArray
    /// ```
    ///
    /// Reverse `array`.
    ///
    /// Usage:
    /// ```
    /// # use bitcoin_cash::Opcode::*;
    /// # struct Params;
    /// # #[bitcoin_cash::script(DemoInputs)]
    /// # fn demo(_: Params) {
    /// let array = b"BitcoinCash";
    ///
    /// // reverse array
    /// let reversed = OP_REVERSEBYTES(array);
    ///
    /// let expected = b"hsaCnioctiB";
    /// OP_EQUALVERIFY(reversed, expected);
    /// # }
    /// ```
    OP_REVERSEBYTES = 0xbc,

    /// The first op_code value after all defined opcodes
    FIRST_UNDEFINED_OP_VALUE,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = self.into();
        write!(f, "{}", s)
    }
}

/// Internal module used for type checking.
///
/// Functions only push mock data.
pub mod func {
    #![allow(non_snake_case)]
    #![allow(unused_variables)]

    use super::*;

    pub fn FIRST<T>(item1: T, item2: T) -> T {
        item1
    }

    pub fn SECOND<T>(item1: T, item2: T) -> T {
        item2
    }

    pub fn OP_1NEGATE() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_0() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_1() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_2() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_3() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_4() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_5() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_6() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_7() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_8() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_9() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_10() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_11() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_12() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_13() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_14() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_15() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_16() -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }

    pub fn OP_NOP() {}
    pub fn OP_NOP1() {}
    pub fn OP_NOP4() {}
    pub fn OP_NOP5() {}
    pub fn OP_NOP6() {}
    pub fn OP_NOP7() {}
    pub fn OP_NOP8() {}
    pub fn OP_NOP9() {}
    pub fn OP_NOP10() {}

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
        BitcoinInteger(Integer::ZERO)
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
    pub fn OP_RETURN() {}

    pub fn OP_CAT(left: BitcoinByteArray, right: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_SPLIT(
        array: BitcoinByteArray,
        split_idx: BitcoinInteger,
    ) -> (BitcoinByteArray, BitcoinByteArray) {
        (
            BitcoinByteArray(b"left".as_ref().into()),
            BitcoinByteArray(b"right".as_ref().into()),
        )
    }
    pub fn OP_NUM2BIN(num: BitcoinInteger, byte_size: BitcoinInteger) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_BIN2NUM(array: BitcoinByteArray) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_SIZE(array: BitcoinByteArray) -> (BitcoinByteArray, BitcoinInteger) {
        (array, BitcoinInteger(Integer::ZERO))
    }
    pub fn OP_AND(array1: BitcoinByteArray, array2: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_OR(array1: BitcoinByteArray, array2: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_XOR(array1: BitcoinByteArray, array2: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_EQUAL<T>(item1: T, item2: T) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_EQUALVERIFY<T>(item1: T, item2: T) {}

    pub fn OP_1ADD(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_1SUB(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_NEGATE(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_ABS(num: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_NOT(boolean: BitcoinBoolean) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_0NOTEQUAL(num: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_ADD(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_SUB(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_DIV(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_MOD(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_BOOLAND(boolean1: BitcoinBoolean, boolean2: BitcoinBoolean) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_BOOLOR(boolean1: BitcoinBoolean, boolean2: BitcoinBoolean) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_NUMEQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_NUMEQUALVERIFY(num1: BitcoinInteger, num2: BitcoinInteger) {}
    pub fn OP_NUMNOTEQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_LESSTHAN(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_GREATERTHAN(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_LESSTHANOREQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_GREATERTHANOREQUAL(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_MIN(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_MAX(num1: BitcoinInteger, num2: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_WITHIN(
        num1: BitcoinInteger,
        num_min: BitcoinInteger,
        num_max: BitcoinInteger,
    ) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_RIPEMD160(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_SHA1(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_SHA256(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_HASH160(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_HASH256(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
    pub fn OP_CODESEPARATOR() {}
    pub fn OP_CHECKSIG(sig: BitcoinByteArray, pubkey: BitcoinByteArray) -> BitcoinBoolean {
        BitcoinBoolean(true)
    }
    pub fn OP_CHECKSIGVERIFY(sig: BitcoinByteArray, pubkey: BitcoinByteArray) {}
    pub fn OP_CHECKLOCKTIMEVERIFY(locktime: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
    }
    pub fn OP_CHECKSEQUENCEVERIFY(sequence: BitcoinInteger) -> BitcoinInteger {
        BitcoinInteger(Integer::ZERO)
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
    pub fn OP_REVERSEBYTES(array: BitcoinByteArray) -> BitcoinByteArray {
        BitcoinByteArray(b"MOCK".as_ref().into())
    }
}

impl Opcode {
    pub fn is_disabled(self) -> bool {
        use Opcode::*;
        match self {
            OP_RESERVED | OP_RESERVED1 | OP_RESERVED2 | OP_MUL | OP_2MUL | OP_2DIV | OP_INVERT
            | OP_LSHIFT | OP_RSHIFT | OP_VER | OP_VERIF | OP_VERNOTIF => true,
            _ => false,
        }
    }

    pub fn retains_input(self) -> bool {
        use Opcode::*;
        match self {
            OP_SIZE | OP_CHECKLOCKTIMEVERIFY | OP_CHECKSEQUENCEVERIFY => true,
            _ => false,
        }
    }

    pub fn behavior(self) -> OpcodeBehavior {
        use DataType::*;
        use Opcode::*;
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
                &[Observed, Observed, Untouched, Untouched, Added, Added],
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
                &[MovedIndirectly, MovedIndirectly, Moved, Moved],
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
                &[],
            ),
            OP_REVERSEBYTES => u(&[ByteArray(None)], &[ByteArray(None)], &[Changed]),
            OP_0 | OP_1NEGATE | OP_1 | OP_2 | OP_3 | OP_4 | OP_5 | OP_6 | OP_7 | OP_8 | OP_9
            | OP_10 | OP_11 | OP_12 | OP_13 | OP_14 | OP_15 | OP_16 => u(&[], &[Integer], &[Added]),

            OP_IF => u(&[T], &[], &[]),
            OP_ELSE => u(&[], &[], &[]),
            OP_ENDIF => u(&[], &[], &[]),

            OP_TOALTSTACK => u(&[T], &[], &[]),
            OP_FROMALTSTACK => u(&[], &[T], &[Added]),

            OP_CHECKLOCKTIMEVERIFY => u(&[Integer], &[Integer], &[Observed]),
            OP_CHECKSEQUENCEVERIFY => u(&[Integer], &[Integer], &[Observed]),

            OP_IFDUP | OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {
                panic!("Opcode behavior cannot be expressed in OpcodeBehavior")
            }

            opcode if opcode.is_disabled() => panic!("Opcode is disabled"),

            _ => u(&[], &[], &[]),
        }
    }
}
