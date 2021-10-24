pub struct SourceFile<'a> {
    src: &'a str,
    lines: Vec<usize>,
}

impl<'a> SourceFile<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut lines = vec![];
        for (pos, c) in src.char_indices() {
            if c == '\n' {
                lines.push(pos);
            }
        }

        SourceFile { src, lines }
    }

    pub fn loc_for_byte_pos(&self, pos: usize) -> (usize, usize) {
        let mut line_start = 0;
        for (line, &next_newline) in self.lines.iter().enumerate() {
            if next_newline >= pos {
                let char_pos = self.src[line_start..pos].chars().count() + 1;
                return (line + 1, char_pos);
            }
            line_start = next_newline + 1;
        }

        let char_pos = self.src[line_start..pos].chars().count() + 1;
        (self.lines.len() + 1, char_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loc_for_byte_pos() {
        let t = SourceFile::new("á\néf\n\ng");

        assert_eq!(t.loc_for_byte_pos(0), (1, 1)); // á
        assert_eq!(t.loc_for_byte_pos(2), (1, 2)); // \n
        assert_eq!(t.loc_for_byte_pos(3), (2, 1)); // é
        assert_eq!(t.loc_for_byte_pos(5), (2, 2)); // f
        assert_eq!(t.loc_for_byte_pos(8), (4, 1)); // g
    }
}
