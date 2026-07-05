// This module defines the schema matching (or trying to match?) Qt's XSD
// Eventually when a proper Rust code generator exists it would be great to use that instead.
// For now they can't handle Qt's semi-weird XSD.

// https://doc.qt.io/qt-6/linguist-ts-file-format.html
pub mod parse_error;

mod ts_parser;
mod ts_node;
mod ts_bytes;
mod context_node;
mod message_node;
mod translation_type;
mod yesno;
mod translation_node;
mod location_node;
mod dependency_node;
mod numerus_form_node;