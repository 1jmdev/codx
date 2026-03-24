mod error;
mod fold;
mod highlight;
mod indent;
mod language;
mod layer;
mod markdown_injection;
mod registry;

pub use error::SyntaxError;
pub use fold::{compute_folds, FoldRange};
pub use highlight::{spans_for_line, HighlightSpan};
pub use indent::compute_indent;
pub use language::{language_for_name, language_for_path, LanguageId};
pub use layer::SyntaxLayer;
pub use markdown_injection::markdown_code_block_spans_for_line;
pub use registry::LanguageRegistry;
