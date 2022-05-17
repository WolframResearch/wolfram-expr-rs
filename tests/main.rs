use wolfram_expr::Expr;

#[test]
fn test_curry() {
    // Normal@BinarySerialize[Sin[1]]
    let v = Expr::function("System`Sin", vec![Expr::from(1)]);
    assert_eq!(v.as_wxf(), [
        56, 58, 102, 1, 115, 10, 83, 121, 115, 116, 101, 109, 96, 83, 105, 110, 76, 1, 0,
        0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(v.to_string(), "System`Sin[1]");
    // Normal@BinarySerialize[Sin[1][2]]
    let v = Expr::normal(v, vec![Expr::from(2)]);
    assert_eq!(v.as_wxf(), [
        56, 58, 102, 1, 102, 1, 115, 10, 83, 121, 115, 116, 101, 109, 96, 83, 105, 110,
        76, 1, 0, 0, 0, 0, 0, 0, 0, 76, 2, 0, 0, 0, 0, 0, 0, 0
    ]);
    assert_eq!(v.to_string(), "System`Sin[1][2]");
    // Normal@BinarySerialize[Sin[1][2][3]]
    let v = Expr::normal(v, vec![Expr::from(3)]);
    assert_eq!(v.as_wxf(), [
        56, 58, 102, 1, 102, 1, 102, 1, 115, 10, 83, 121, 115, 116, 101, 109, 96, 83,
        105, 110, 76, 1, 0, 0, 0, 0, 0, 0, 0, 76, 2, 0, 0, 0, 0, 0, 0, 0, 76, 3, 0, 0, 0,
        0, 0, 0, 0
    ]);
    assert_eq!(v.to_string(), "System`Sin[1][2][3]");
}
