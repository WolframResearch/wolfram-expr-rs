use serde::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};

use wolfram_expr::WolframSerializer;

pub struct TestEmpty {}

#[derive(Serialize, Deserialize)]
pub struct TestBody<'a> {
    bool: bool,
    int: i64,
    str: &'a str,
    string: String,
    bytes: &'a [u8],
    vector: Vec<u8>,
    point1: (f64, f64),
    point2: Point2D,
}

#[derive(Serialize, Deserialize)]
pub struct Point2D {
    x: f64,
    y: f64,
}

#[test]
fn test_serialize() {
    let serializer = WolframSerializer { readable: true };
    let str = "ref string";
    let bytes = &[1, 2, 3, 4, 5];
    let input = TestBody {
        bool: true,
        int: 42,
        str,
        string: "own string".to_string(),
        bytes,
        vector: bytes.to_vec(),
        point1: (1.0, 2.0),
        point2: Point2D { x: 3.0, y: 4.0 },
    };
    let out = input.serialize(&serializer).unwrap();
    println!("{:?}", out);
}
