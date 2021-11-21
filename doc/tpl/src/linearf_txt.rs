use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "linearf.txt.stpl")]
struct LinearfTxt {}

pub fn fetch_context() -> impl TemplateOnce { LinearfTxt {} }
