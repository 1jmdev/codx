#[derive(Debug, Clone, Default)]
pub struct ProgressState {
    pub title: String,
    pub message: String,
    pub percentage: Option<u32>,
    pub done: bool,
}

impl ProgressState {
    pub fn label(&self) -> String {
        if self.title.is_empty() && self.message.is_empty() {
            return String::new();
        }

        let mut text = String::new();
        if !self.title.is_empty() {
            text.push_str(&self.title);
        }
        if !self.message.is_empty() {
            if !text.is_empty() {
                text.push_str(": ");
            }
            text.push_str(&self.message);
        }
        if let Some(percentage) = self.percentage {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&format!("{percentage}%"));
        }

        text
    }
}
