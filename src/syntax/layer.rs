use tree_sitter::{Parser, Tree};

use crate::syntax::{LanguageId, SyntaxError};

pub struct SyntaxLayer {
    language_id: Option<LanguageId>,
    parser: Parser,
    tree: Option<Tree>,
    dirty: bool,
}

impl SyntaxLayer {
    pub fn new(language_id: Option<LanguageId>) -> Self {
        let mut parser = Parser::new();
        if let Some(id) = language_id {
            let lang = id.ts_language();
            let _ = parser.set_language(&lang);
        }
        Self {
            language_id,
            parser,
            tree: None,
            dirty: true,
        }
    }

    pub fn reparse(&mut self, source: &[u8]) -> Result<(), SyntaxError> {
        if self.language_id.is_none() {
            self.dirty = false;
            return Ok(());
        }
        let old_tree = self.tree.as_ref();
        match self.parser.parse(source, old_tree) {
            Some(tree) => {
                self.tree = Some(tree);
                self.dirty = false;
                Ok(())
            }
            None => Err(SyntaxError::ParseFailed),
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub fn language_id(&self) -> Option<LanguageId> {
        self.language_id
    }
}
