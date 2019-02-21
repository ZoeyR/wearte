use crate::parser::ws;

// https://github.com/djc/askama/blob/master/askama_derive/src/generator.rs#L1189-L1232
pub(super) struct Buffer {
    // The buffer to generate the code into
    pub(super) buf: String,
    // The current level of indentation (in spaces)
    indent: u8,
    // Whether the output buffer is currently at the start of a line
    start: bool,
}

impl Buffer {
    pub(super) fn new(indent: u8) -> Self {
        Self {
            buf: String::new(),
            indent,
            start: true,
        }
    }

    pub(super) fn writeln(&mut self, mut s: &str) {
        for (i, b) in s.as_bytes().iter().enumerate() {
            if !ws(*b) {
                if *b == b'}' {
                    s = s.get(i..).unwrap()
                }
                break;
            }
        }

        if !s.is_empty() {
            if s.as_bytes()[0] == b'}' {
                self.dedent();
            }

            self.write(s);

            if *s.as_bytes().last().unwrap() == b'{' {
                self.indent();
            }
        }

        self.buf.push('\n');
        self.start = true;
    }

    pub(super) fn write(&mut self, s: &str) {
        if self.start {
            for _ in 0..(self.indent * 4) {
                self.buf.push(' ');
            }
            self.start = false;
        }
        self.buf.push_str(s);
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent == 0 {
            panic!("dedent() called while indentation == 0");
        }
        self.indent -= 1;
    }
}
