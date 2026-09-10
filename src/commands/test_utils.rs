use crate::parser::serializer::serialize;
use crate::parser::ts_parser::TsDocument;
use std::io::BufWriter;
use std::path::PathBuf;

#[cfg(test)]
pub fn test_file_content_as_string(filename: &str) -> String {
    std::fs::read_to_string(PathBuf::new().join("./test_data").join(filename))
        .expect("Data file is readable")
}

#[cfg(test)]
pub fn test_file_content_bytes(filename: &str) -> Vec<u8> {
    std::fs::read_to_string(PathBuf::new().join("./test_data").join(filename))
        .map(|s| s.into_bytes())
        .expect("Data file is readable")
}

#[cfg(test)]
pub fn serialize_to_string(doc: &TsDocument) -> String {
    let mut buf = BufWriter::new(Vec::<u8>::new());
    serialize(&mut buf, doc).expect("Sorted data can be serialized");
    String::from_utf8(buf.into_inner().expect("Inner buffer is valid"))
        .expect("Buffer is valid utf-8")
}
