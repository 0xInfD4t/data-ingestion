pub mod detector;
pub mod reader_csv;
pub mod reader_json;
pub mod reader_json_schema;
pub mod reader_xml;
pub mod reader_xsd;
pub mod reader_yaml;
pub mod traits;
pub(crate) mod type_inference;

pub use detector::FormatDetector;
pub use traits::{FormatHint, FormatReader};
pub use crate::ir::model::SourceFormat;
