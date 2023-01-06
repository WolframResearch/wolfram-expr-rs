use serde::Serializer;
use serde_derive::{Deserialize, Serialize};
use wolfram_expr::{Expr, WolframSerializer};

pub struct TestEmpty {

}

#[derive(Serialize, Deserialize)]
pub struct TestBody<'a> {
    str: &'a str,
    string: String
}

#[test]
fn test() {
    let expr = Expr::null();

    let mut serializer = WolframSerializer {
        readable: true,
    };
    let out = serializer.serialize_seq(vec![1, 2, 3]);
    println!("{:?}", out);
}