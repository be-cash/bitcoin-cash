use bitcoin_cash_script::{ByteArray, Op, OpcodeType::*, Ops, TaggedOp};

#[test]
fn test_adding() {
    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(_: ()) {
        6;
        5;
        OP_ADD;
    }
    Inputs {};
    assert_eq!(
        script(()).tagged_ops(),
        &[
            TaggedOp {
                src: "6".into(),
                op: Op::PushInteger(6),
                input_names: None,
                output_names: None,
            },
            TaggedOp {
                src: "5".into(),
                op: Op::PushInteger(5),
                input_names: None,
                output_names: None,
            },
            TaggedOp {
                src: "OP_ADD".into(),
                op: Op::Code(OP_ADD),
                input_names: None,
                output_names: None,
            },
        ],
    );
}

#[test]
fn test_catting() {
    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(_: ()) {
        b"A";
        b"B";
        OP_TUCK;
        OP_CAT;
        OP_CAT;
    }
    assert_eq!(
        script(()).ops().as_ref(),
        &[
            Op::PushByteArray(ByteArray::from_slice(b"A")),
            Op::PushByteArray(ByteArray::from_slice(b"B")),
            Op::Code(OP_TUCK),
            Op::Code(OP_CAT),
            Op::Code(OP_CAT),
        ],
    );
}

#[test]
fn test_inputs() {
    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(_: (), a: [u8; 1], b: [u8; 1]) {
        OP_CAT;
    }
    assert_eq!(script(()).ops().as_ref(), &[Op::Code(OP_CAT)],);
    assert_eq!(
        Inputs {
            a: b"A".clone(),
            b: b"B".clone(),
        }
        .ops()
        .as_ref(),
        &[
            Op::PushByteArray(ByteArray::from_slice(b"A")),
            Op::PushByteArray(ByteArray::from_slice(b"B")),
        ],
    );
}

#[test]
fn test_let() {
    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(_: (), a: i32, b: i32) {
        let c = OP_ADD(a, b);
        let (d, e) = OP_DUP(c);
        let f = OP_DIV(d, e);
        {
            let (g, h) = OP_DUP(f);
            OP_SUB(g, h);
        }
    }

    assert_eq!(
        script(()).tagged_ops(),
        &[
            TaggedOp {
                src: "let c = OP_ADD (a, b) ;".into(),
                op: Op::Code(OP_ADD),
                input_names: Some(vec!["a".into(), "b".into()]),
                output_names: Some(vec!["c".into()]),
            },
            TaggedOp {
                src: "let (d, e) = OP_DUP (c) ;".into(),
                op: Op::Code(OP_DUP),
                input_names: Some(vec!["c".into()]),
                output_names: Some(vec!["d".into(), "e".into()]),
            },
            TaggedOp {
                src: "let f = OP_DIV (d, e) ;".into(),
                op: Op::Code(OP_DIV),
                input_names: Some(vec!["d".into(), "e".into()]),
                output_names: Some(vec!["f".into()]),
            },
            TaggedOp {
                src: "let (g, h) = OP_DUP (f) ;".into(),
                op: Op::Code(OP_DUP),
                input_names: Some(vec!["f".into()]),
                output_names: Some(vec!["g".into(), "h".into()]),
            },
            TaggedOp {
                src: "OP_SUB (g, h)".into(),
                op: Op::Code(OP_SUB),
                input_names: Some(vec!["g".into(), "h".into()]),
                output_names: None,
            },
        ],
    );

    assert_eq!(
        script(()).ops().as_ref(),
        &[
            Op::Code(OP_ADD),
            Op::Code(OP_DUP),
            Op::Code(OP_DIV),
            Op::Code(OP_DUP),
            Op::Code(OP_SUB),
        ],
    );
    assert_eq!(
        Inputs { a: 5, b: 6 }.ops().as_ref(),
        &[Op::PushInteger(5), Op::PushInteger(6),],
    );
}

#[test]
fn test_if() {
    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(_: (), a: i32, b: bool) {
        OP_IF(b);
        let _x = OP_1ADD(a);
        OP_ELSE;
        let _x = OP_1SUB(a);
        OP_ENDIF;
        let y = 3;
        OP_DIV(_x, y);
    }
    assert_eq!(
        script(()).ops().as_ref(),
        &[
            Op::Code(OP_IF),
            Op::Code(OP_1ADD),
            Op::Code(OP_ELSE),
            Op::Code(OP_1SUB),
            Op::Code(OP_ENDIF),
            Op::PushInteger(3),
            Op::Code(OP_DIV),
        ],
    );
    assert_eq!(
        Inputs { a: 5, b: true }.ops().as_ref(),
        &[Op::PushInteger(5), Op::PushBoolean(true),],
    );
}

#[test]
fn test_params() {
    struct Params {
        p1: i32,
        p2: Vec<u8>,
        p3: i32,
    }

    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(params: &Params, a: i32, b: bool, c: [u8; 32]) {
        let p2 = params.p2;
        let c = OP_CAT(c, p2);
        let c = OP_BIN2NUM(c);
        let p1 = params.p1;
        let sum = OP_ADD(c, p1);
        let (sum, b) = OP_SWAP(b, sum);
        OP_IF(b);
        {
            let sum2 = OP_ADD(a, sum);
            let n = 4;
        }
        OP_ELSE;
        {
            let sum2 = OP_SUB(a, sum);
            let n = 4;
        }
        OP_ENDIF;
        let sum2 = OP_NUM2BIN(sum2, n);
        let sum2 = OP_BIN2NUM(sum2);
        let limit = params.p3;
        OP_GREATERTHAN(sum2, limit);
    }

    let params = Params {
        p1: 3,
        p2: b"A".to_vec(),
        p3: 6,
    };

    assert_eq!(
        script(&params).ops().as_ref(),
        &[
            Op::PushByteArray(params.p2.into()),
            Op::Code(OP_CAT),
            Op::Code(OP_BIN2NUM),
            Op::PushInteger(params.p1),
            Op::Code(OP_ADD),
            Op::Code(OP_SWAP),
            Op::Code(OP_IF),
            Op::Code(OP_ADD),
            Op::PushInteger(4),
            Op::Code(OP_ELSE),
            Op::Code(OP_SUB),
            Op::PushInteger(4),
            Op::Code(OP_ENDIF),
            Op::Code(OP_NUM2BIN),
            Op::Code(OP_BIN2NUM),
            Op::PushInteger(params.p3),
            Op::Code(OP_GREATERTHAN),
        ],
    );
}

#[test]
fn test_depth_of() {
    #[bitcoin_cash_script_macro::script(Inputs)]
    fn script(_: ()) {
        let _a = 6;
        let _b = 5;
        let _c = 4;
        let _depth = depth_of(_a);
        OP_PICK(_depth);
    }

    assert_eq!(
        script(()).ops().as_ref(),
        &[
            Op::PushInteger(6),
            Op::PushInteger(5),
            Op::PushInteger(4),
            Op::PushInteger(2),
            Op::Code(OP_PICK),
        ],
    );
}
