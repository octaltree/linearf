use sailfish::TemplateOnce;
use std::path::Path;

fn main() { write("../linearf.txt", tpl::linearf_txt::fetch_context()); }

fn write<P: AsRef<Path>>(path: P, ctx: impl TemplateOnce) {
    let mut buf = sailfish::runtime::Buffer::new();
    ctx.render_once_to(&mut buf).unwrap();
    buf.push('\n');
    std::fs::write(path, buf.as_str()).unwrap();
}
