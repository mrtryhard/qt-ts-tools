use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn read_test_file(filename: &str) -> String {
    let mut buf = String::new();
    let _ = File::open(PathBuf::new().join("./test_data").join(filename))
        .expect("Data file is readable")
        .read_to_string(&mut buf)
        .expect("Output to string");
    buf.replace('\r', "")
}
