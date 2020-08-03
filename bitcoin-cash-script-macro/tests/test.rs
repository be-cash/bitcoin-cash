use bitcoin_cash::{ByteArray, Op, Opcode::*, Ops, TaggedOp};
use pretty_assertions::assert_eq;

#[test]
fn test_adding() {
    let line = line!() + 2;
    #[bitcoin_cash::script(Inputs)]
    fn script(_: ()) {
        6;
        5;
        OP_ADD;
    }
    Inputs {};
    let file_name = file!();
    assert_eq!(
        script(()).ops().as_ref(),
        &[
            TaggedOp {
                src_code: vec![
                    (30, "6;".into()),
                    (40, "6;".into()),
                    (60, "6;".into()),
                    (80, "6;".into()),
                ],
                src_file: file_name.into(),
                src_line: line + 1,
                src_column: 9,
                op: Op::PushInteger(6),
                pushed_names: Some(vec![None]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (30, "5;".into()),
                    (40, "5;".into()),
                    (60, "5;".into()),
                    (80, "5;".into()),
                ],
                src_file: file_name.into(),
                src_line: line + 2,
                src_column: 9,
                op: Op::PushInteger(5),
                pushed_names: Some(vec![None]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (30, "OP_ADD;".into()),
                    (40, "OP_ADD;".into()),
                    (60, "OP_ADD;".into()),
                    (80, "OP_ADD;".into()),
                ],
                src_file: file_name.into(),
                src_line: line + 3,
                src_column: 9,
                op: Op::Code(OP_ADD),
                pushed_names: Some(vec![None]),
                alt_pushed_names: Some(vec![]),
            },
        ],
    );
}

#[test]
fn test_catting() {
    #[bitcoin_cash::script(Inputs)]
    fn script(_: ()) {
        b"A";
        b"B";
        OP_TUCK;
        OP_CAT;
        OP_CAT;
    }
    assert_eq!(
        script(())
            .ops()
            .into_iter()
            .map(|op| &op.op)
            .collect::<Vec<_>>()
            .as_slice(),
        &[
            &ByteArray::from_slice_unnamed(b"A").into(),
            &ByteArray::from_slice_unnamed(b"B").into(),
            &Op::Code(OP_TUCK),
            &Op::Code(OP_CAT),
            &Op::Code(OP_CAT),
        ],
    );
}

#[test]
fn test_inputs() {
    #[bitcoin_cash::script(Inputs)]
    fn script(_: (), a: [u8; 1], b: [u8; 1]) {
        OP_CAT;
    }
    assert_eq!(
        script(()).script_ops().collect::<Vec<_>>(),
        vec![&Op::Code(OP_CAT)]
    );
    assert_eq!(
        Inputs {
            a: b"A".clone(),
            b: b"B".clone(),
        }
        .ops()
        .iter()
        .map(|op| &op.op)
        .collect::<Vec<_>>(),
        vec![
            &ByteArray::from_slice_unnamed(b"A").into(),
            &ByteArray::from_slice_unnamed(b"B").into(),
        ],
    );
}

#[test]
fn test_let() {
    struct Params {
        hyperfine_structure: i32,
    }

    let line = line!() + 2;
    #[bitcoin_cash::script(Inputs)]
    fn script(params: &Params, alpha: i32, beta: i32) {
        let hyperfine_structure = params.hyperfine_structure * 1000;
        let circumference = OP_ADD(beta, hyperfine_structure);
        let relative_velocity = OP_ADD(alpha, circumference);
        let (relative_velocity1, relative_velocity2) = OP_DUP(relative_velocity);
        let f = OP_DIV(relative_velocity1, relative_velocity2);
        {
            let (g, __) = OP_DUP(f);
            OP_SUB(g, __);
        }
    }
    let file_name = file!();

    let params = Params {
        hyperfine_structure: 1337,
    };

    assert_eq!(
        script(&params).ops().as_ref(),
        &[
            TaggedOp {
                src_code: vec![
                    (
                        30,
                        "\
let hyperfine_structure =
    params
        .hyperfine_structure
        * 1000;"
                            .into()
                    ),
                    (
                        40,
                        "\
let hyperfine_structure =
    params.hyperfine_structure * 1000;"
                            .into()
                    ),
                    (
                        60,
                        "let hyperfine_structure = params.hyperfine_structure * 1000;".into()
                    ),
                    (
                        80,
                        "let hyperfine_structure = params.hyperfine_structure * 1000;".into()
                    ),
                ],
                src_file: file_name.into(),
                src_line: line + 1,
                src_column: 35,
                op: Op::PushInteger(1337 * 1000),
                pushed_names: Some(vec![Some("hyperfine_structure".into())]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (
                        30,
                        "\
let circumference = OP_ADD(
    beta,
    hyperfine_structure,
);"
                        .into()
                    ),
                    (
                        40,
                        "\
let circumference =
    OP_ADD(beta, hyperfine_structure);"
                            .into()
                    ),
                    (
                        60,
                        "let circumference = OP_ADD(beta, hyperfine_structure);".into()
                    ),
                    (
                        80,
                        "let circumference = OP_ADD(beta, hyperfine_structure);".into()
                    ),
                ],
                src_file: file_name.into(),
                src_line: line + 2,
                src_column: 29,
                op: Op::Code(OP_ADD),
                pushed_names: Some(vec![Some("circumference".into())]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (
                        30,
                        "\
let relative_velocity =
    OP_ADD(
        alpha,
        circumference,
    );"
                        .into()
                    ),
                    (
                        40,
                        "\
let relative_velocity =
    OP_ADD(alpha, circumference);"
                            .into()
                    ),
                    (
                        60,
                        "let relative_velocity = OP_ADD(alpha, circumference);".into()
                    ),
                    (
                        80,
                        "let relative_velocity = OP_ADD(alpha, circumference);".into()
                    ),
                ],
                src_file: file_name.into(),
                src_line: line + 3,
                src_column: 33,
                op: Op::Code(OP_ADD),
                pushed_names: Some(vec![Some("relative_velocity".into())]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (
                        30,
                        "\
let (
    relative_velocity1,
    relative_velocity2,
) = OP_DUP(relative_velocity);"
                            .into()
                    ),
                    (
                        40,
                        "\
let (
    relative_velocity1,
    relative_velocity2,
) = OP_DUP(relative_velocity);"
                            .into()
                    ),
                    (
                        60,
                        "\
let (relative_velocity1, relative_velocity2) =
    OP_DUP(relative_velocity);"
                            .into()
                    ),
                    (
                        80,
                        "let (relative_velocity1, relative_velocity2) = OP_DUP(relative_velocity);"
                            .into()
                    ),
                ],
                src_file: file_name.into(),
                src_line: line + 4,
                src_column: 56,
                op: Op::Code(OP_DUP),
                pushed_names: Some(vec![
                    Some("relative_velocity1".into()),
                    Some("relative_velocity2".into())
                ]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (
                        30,
                        "\
let f = OP_DIV(
    relative_velocity1,
    relative_velocity2,
);"
                        .into()
                    ),
                    (
                        40,
                        "\
let f = OP_DIV(
    relative_velocity1,
    relative_velocity2,
);"
                        .into()
                    ),
                    (
                        60,
                        "let f = OP_DIV(relative_velocity1, relative_velocity2);".into()
                    ),
                    (
                        80,
                        "let f = OP_DIV(relative_velocity1, relative_velocity2);".into()
                    ),
                ],
                src_file: file_name.into(),
                src_line: line + 5,
                src_column: 17,
                op: Op::Code(OP_DIV),
                pushed_names: Some(vec![Some("f".into())]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (30, "let (g, __) = OP_DUP(f);".into()),
                    (40, "let (g, __) = OP_DUP(f);".into()),
                    (60, "let (g, __) = OP_DUP(f);".into()),
                    (80, "let (g, __) = OP_DUP(f);".into()),
                ],
                src_file: file_name.into(),
                src_line: line + 7,
                src_column: 27,
                op: Op::Code(OP_DUP),
                pushed_names: Some(vec![Some("g".into()), Some("f".into())]),
                alt_pushed_names: Some(vec![]),
            },
            TaggedOp {
                src_code: vec![
                    (30, "OP_SUB(g, __);".into()),
                    (40, "OP_SUB(g, __);".into()),
                    (60, "OP_SUB(g, __);".into()),
                    (80, "OP_SUB(g, __);".into()),
                ],
                src_file: file_name.into(),
                src_line: line + 8,
                src_column: 13,
                op: Op::Code(OP_SUB),
                pushed_names: Some(vec![None]),
                alt_pushed_names: Some(vec![]),
            },
        ],
    );

    assert_eq!(
        &script(&params).script_ops().collect::<Vec<_>>(),
        &[
            &Op::PushInteger(1337 * 1000),
            &Op::Code(OP_ADD),
            &Op::Code(OP_ADD),
            &Op::Code(OP_DUP),
            &Op::Code(OP_DIV),
            &Op::Code(OP_DUP),
            &Op::Code(OP_SUB),
        ],
    );
    assert_eq!(
        Inputs { alpha: 5, beta: 6 }
            .ops()
            .iter()
            .map(|op| &op.op)
            .collect::<Vec<_>>(),
        &[&Op::PushInteger(5), &Op::PushInteger(6)],
    );
}

#[test]
fn test_if() {
    #[bitcoin_cash::script(Inputs)]
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
        &script(()).script_ops().collect::<Vec<_>>(),
        &[
            &Op::Code(OP_IF),
            &Op::Code(OP_1ADD),
            &Op::Code(OP_ELSE),
            &Op::Code(OP_1SUB),
            &Op::Code(OP_ENDIF),
            &Op::PushInteger(3),
            &Op::Code(OP_DIV),
        ],
    );
    assert_eq!(
        Inputs { a: 5, b: true }
            .ops()
            .iter()
            .map(|op| &op.op)
            .collect::<Vec<_>>(),
        &[&Op::PushInteger(5), &Op::PushBoolean(true),],
    );
}

#[test]
fn test_params() {
    struct Params {
        p1: i32,
        p2: Vec<u8>,
        p3: i32,
    }

    #[bitcoin_cash::script(Inputs)]
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
            let _n = 4;
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
        &script(&params).script_ops().collect::<Vec<_>>(),
        &[
            &Op::PushByteArray {
                array: params.p2.into(),
                is_minimal: true,
            },
            &Op::Code(OP_CAT),
            &Op::Code(OP_BIN2NUM),
            &Op::PushInteger(params.p1),
            &Op::Code(OP_ADD),
            &Op::Code(OP_SWAP),
            &Op::Code(OP_IF),
            &Op::Code(OP_ADD),
            &Op::PushInteger(4),
            &Op::Code(OP_ELSE),
            &Op::Code(OP_SUB),
            &Op::PushInteger(4),
            &Op::Code(OP_ENDIF),
            &Op::Code(OP_NUM2BIN),
            &Op::Code(OP_BIN2NUM),
            &Op::PushInteger(params.p3),
            &Op::Code(OP_GREATERTHAN),
        ],
    );
}

#[test]
fn test_depth_of() {
    #[bitcoin_cash::script(Inputs)]
    fn script(_: ()) {
        let _a = 6;
        let _b = 5;
        let _c = 4;
        let _depth = depth_of(_a);
        OP_PICK(_depth);
        OP_1ADD(_a);
    }

    assert_eq!(
        &script(()).script_ops().collect::<Vec<_>>(),
        &[
            &Op::PushInteger(6),
            &Op::PushInteger(5),
            &Op::PushInteger(4),
            &Op::PushInteger(2),
            &Op::Code(OP_PICK),
            &Op::Code(OP_1ADD),
        ],
    );
}

#[test]
fn test_attributes() {
    #[bitcoin_cash::script(Inputs, A = "!p1", B = "p1")]
    fn script(_: (), #[variant(A)] a: i32, #[variant(A, B)] b: i32, c: i32) {
        let p1 = OP_0NOTEQUAL(c);
        OP_IF(p1);
        {
            let c = OP_1ADD(b);
        }
        OP_ELSE;
        {
            let c = OP_SUB(a, b);
        }
        OP_ENDIF;
        let d = OP_1SUB(c);
        let p2 = OP_0NOTEQUAL(d);
        OP_VERIFY(p2);
    }

    Inputs::A { a: 12, b: 5, c: 4 };

    Inputs::B { b: 3, c: 0 };
}

#[test]
fn test_generics() {
    #[bitcoin_cash::script(Inputs)]
    fn script(_: (), a: ByteArray) {
        let _4 = 4;
        let (b, c) = OP_SPLIT(a, _4);
    }
    assert_eq!(
        &script(()).script_ops().collect::<Vec<_>>(),
        &[&Op::PushInteger(4), &Op::Code(OP_SPLIT)],
    );
    assert_eq!(
        Inputs {
            a: ByteArray::from_slice("a", b"")
        }
        .ops()
        .iter()
        .map(|op| &op.op)
        .collect::<Vec<_>>(),
        vec![&Op::PushByteArray {
            array: ByteArray::from_slice("a", b""),
            is_minimal: true,
        }],
    );
}

#[test]
fn test_variants() {
    #[bitcoin_cash::script(Inputs, A = "!p1", B = "p1")]
    fn script(_: (), #[variant(A)] a: ByteArray, #[variant(A, B)] b: ByteArray, c: ByteArray) {
        let empty_str = b"";
        let p1 = OP_EQUAL(c, empty_str);
        OP_IF(p1);
        {
            let suffix = b"bla";
            let c = OP_CAT(b, suffix);
        }
        OP_ELSE;
        {
            let c = OP_CAT(a, b);
        }
        OP_ENDIF;
        let _4 = 4;
        let (_x, _y) = OP_SPLIT(c, _4);
    }

    Inputs::A {
        a: ByteArray::from_slice("a", b"tree"),
        b: ByteArray::from_slice("b", b"milk"),
        c: ByteArray::from_slice("c", b"eggs"),
    };

    Inputs::B {
        b: ByteArray::from_slice("b", b"potato"),
        c: ByteArray::from_slice("c", b"pineapple"),
    };
}

#[test]
fn test_placeholder() {
    #[bitcoin_cash::script(Inputs)]
    fn script(_: (), a: i32, b: i32, c: i32) {
        let (__, __, beer) = OP_ROT(a, __, __);
        OP_DROP(beer);
        OP_DROP(c);
        OP_DROP(b);
    }
}

#[test]
fn test_doctest() {
    #[bitcoin_cash::script(Inputs)]
    fn script(
        _: (),
        /// Doctest a
        a: i32,
        /// Doctest b
        b: i32,
        /// Doctest c
        c: i32
    ) {
        let (__, __, beer) = OP_ROT(a, __, __);
        OP_DROP(beer);
        OP_DROP(c);
        OP_DROP(b);
    }
}
