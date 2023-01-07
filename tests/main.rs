use serde::{Serialize, Serializer};
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
    let input = TestBody {
        str: "",
        string: "".to_string(),
    };
    let out = input.serialize(&serializer).unwrap();
    println!("{:?}", out);
}