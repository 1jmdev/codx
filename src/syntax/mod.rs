mod error;
mod fold;
mod highlight;
mod indent;
mod language;
mod layer;
mod registry;

pub use error::SyntaxError;
pub use fold::{FoldRange, compute_folds};
pub use highlight::{HighlightSpan, spans_for_line};
pub use indent::compute_indent;
pub use language::{LanguageId, language_for_path};
pub use layer::SyntaxLayer;
pub use registry::LanguageRegistry;
