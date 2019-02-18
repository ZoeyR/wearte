// https://github.com/djc/askama/blob/master/askama_derive/src/generator.rs#L1189-L1232
// TODO: dedent if begin with whitespace follow be "{"
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

    pub(super) fn writeln(&mut self, s: &str) {
        if s == "}" {
            self.dedent();
        }
        if !s.is_empty() {
            self.write(s);
        }
        self.buf.push('\n');
        if s.ends_with('{') {
            self.indent();
        }
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

    pub(super) fn indent(&mut self) {
        self.indent += 1;
    }

    pub(super) fn dedent(&mut self) {
        if self.indent == 0 {
            panic!("dedent() called while indentation == 0");
        }
        self.indent -= 1;
    }
}
