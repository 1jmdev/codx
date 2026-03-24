use tree_sitter::{InputEdit, Parser, Point, Tree};

use crate::syntax::{LanguageId, SyntaxError};

pub struct SyntaxLayer {
    language_id: Option<LanguageId>,
    parser: Parser,
    tree: Option<Tree>,
    dirty: bool,
    revision: u64,
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
            revision: 0,
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
                self.revision = self.revision.saturating_add(1);
                Ok(())
            }
            None => Err(SyntaxError::ParseFailed),
        }
    }

    pub fn apply_edit(
        &mut self,
        start_byte: usize,
        old_end_byte: usize,
        new_end_byte: usize,
        start_position: Point,
        old_end_position: Point,
        new_end_position: Point,
    ) {
        if let Some(tree) = self.tree.as_mut() {
            tree.edit(&InputEdit {
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            });
        }
        self.dirty = true;
        self.revision = self.revision.saturating_add(1);
    }

    pub fn set_language_id(&mut self, language_id: Option<LanguageId>) -> Result<(), SyntaxError> {
        self.language_id = language_id;
        self.tree = None;

        if let Some(id) = self.language_id {
            let lang = id.ts_language();
            self.parser
                .set_language(&lang)
                .map_err(|error| SyntaxError::LanguageSetFailed(error.to_string()))?;
            self.dirty = true;
        } else {
            self.dirty = false;
        }
        self.revision = self.revision.saturating_add(1);

        Ok(())
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

    pub fn revision(&self) -> u64 {
        self.revision
    }
}
