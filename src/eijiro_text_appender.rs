pub struct TextAppender {
    pub text: String,
    prev_title: Option<String>,
}

impl TextAppender {
    pub fn new(approx_fsize: usize) -> TextAppender {
        TextAppender {
            text: String::with_capacity(approx_fsize),
            prev_title: None,
        }
    }
    pub fn append(&mut self, title: &str, content: &str) {
        if content.starts_with("<→") && content.ends_with(">") {
            return;
        }
        self.prev_title = match title.find("  {") {
            Some(v) => {
                let actual_title = &title[..v];
                let attr = &title[v + 2..];
                self.write_moving_attribute(actual_title, attr, content);
                Some(actual_title.to_string())
            }
            None => {
                self.write_new_whole_line(title, content);
                None
            }
        }
    }
    fn write_new_whole_line(&mut self, title: &str, content: &str) {
        self.text.push_str("\n");
        self.text.push_str(&title);
        self.text.push_str("\t");
        self.text.push_str(&content.replace("■", "\\n"));
    }
    fn write_moving_attribute(&mut self, actual_title: &str, attr: &str, content: &str) {
        match &self.prev_title {
            Some(prev) if prev == actual_title => {
                self.text.push_str("\\n");
            }
            _ => {
                self.text.push_str("\n");
                self.text.push_str(&actual_title);
                self.text.push_str("\t");
            }
        }
        self.text
            .push_str(&attr.replace("{", "【").replace("}", "】"));
        self.text.push_str(&content.replace("■", "\\n"));
    }
}
