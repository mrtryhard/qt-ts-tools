// This module defines the schema matching (or trying to match?) Qt's XSD
// Eventually when a proper Rust code generator exists it would be great to use that instead.
// For now they can't handle Qt's semi-weird XSD.

// https://doc.qt.io/qt-6/linguist-ts-file-format.html
pub mod parse_error;

pub mod context_node;
pub mod dependency_node;
pub mod location_node;
pub mod message_node;
pub mod numerus_form_node;
pub mod translation_node;
pub mod translation_type;
pub mod ts_bytes;
pub mod ts_node;
pub mod ts_parser;
pub mod yesno;
