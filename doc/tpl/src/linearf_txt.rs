use std::path::Path;

use crate::LINEARF_ROOT;
use sailfish::{
    runtime::{Buffer, Render},
    TemplateOnce
};

#[derive(TemplateOnce)]
#[template(path = "linearf.txt.stpl", escape = false)]
struct LinearfTxt {}

pub fn fetch_context() -> impl TemplateOnce { LinearfTxt {} }

fn load_block<P: AsRef<Path>>(
    path: P,
    first_line: &str,
    last_line: &str,
    indent: usize
) -> LoadBlock {
    let path = path.as_ref();
    macro_rules! panic_if_not_found {
        ($m:expr) => {
            match $m {
                None => panic!("{:?}", path),
                Some(x) => x
            }
        };
    }
    let body = std::fs::read_to_string(path).unwrap();
    let mut it = body.lines().enumerate();
    let first = panic_if_not_found!(it.find(|(_, l)| *l == first_line).map(|(i, _)| i));
    let last = if first_line == last_line {
        first
    } else {
        panic_if_not_found!(it.find(|(_, l)| *l == last_line).map(|(i, _)| i))
    };
    LoadBlock {
        body,
        first,
        last,
        indent
    }
}

struct LoadBlock {
    body: String,
    first: usize,
    last: usize,
    indent: usize
}

impl Render for LoadBlock {
    fn render(&self, b: &mut Buffer) -> Result<(), sailfish::RenderError> {
        b.push_str(">\n");
        for (i, l) in self.body.lines().enumerate() {
            if self.first <= i && i <= self.last {
                if !l.is_empty() {
                    for _ in 0..self.indent {
                        b.push_str("		");
                    }
                    b.push_str("  ");
                    b.push_str(&l);
                }
                b.push('\n');
            }
        }
        b.push('<');
        Ok(())
    }
}
