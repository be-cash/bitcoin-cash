#[test]
fn test_adding() {
    #[bitcoin_cash_script_macro::script]
    fn script() -> Vec<bitcoin_cash_script::Op> {
        6;
        5;
        OP_ADD;
    }
    use bitcoin_cash_script::{Op, OpcodeType::*};
    assert_eq!(
        script(),
        vec![Op::PushInteger(6), Op::PushInteger(5), Op::Code(OP_ADD)],
    );
}

#[test]
fn test_catting() {
    #[bitcoin_cash_script_macro::script]
    fn script() -> Vec<bitcoin_cash_script::Op> {
        b"A";
        b"B";
        OP_TUCK;
        OP_CAT;
        OP_CAT;
    }

    use bitcoin_cash_script::{Op, OpcodeType::*};
    assert_eq!(
        script(),
        vec![
            Op::PushByteArray(b"A".to_vec()),
            Op::PushByteArray(b"B".to_vec()),
            Op::Code(OP_TUCK),
            Op::Code(OP_CAT),
            Op::Code(OP_CAT),
        ],
    );
}
