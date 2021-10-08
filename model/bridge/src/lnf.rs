use linearf::*;
use std::sync::Arc;

pub struct Lnf {
    state: Shared<State>,
    source: registry::Source<Self>,
    matcher: registry::Matcher<Self>,
    converter: registry::Converter<Self>,
    rt: AsyncRt
}

impl Lnf {
    pub fn new(state: Shared<State>, rt: AsyncRt) -> Arc<Lnf> {
        Arc::new_cyclic(move |me| -> Lnf {
            let source = registry::Source::new(me.clone());
            let matcher = registry::Matcher::new(me.clone());
            let converter = registry::Converter::new(me.clone());
            Lnf {
                state,
                source,
                matcher,
                converter,
                rt
            }
        })
    }
}

impl Linearf for Lnf {
    type Source = registry::Source<Self>;
    type Matcher = registry::Matcher<Self>;
    type Converter = registry::Converter<Self>;
    fn state(&self) -> &Shared<State> { &self.state }

    fn source(&self) -> &Self::Source { &self.source }
    fn matcher(&self) -> &Self::Matcher { &self.matcher }
    fn converter(&self) -> &Self::Converter { &self.converter }

    fn runtime(&self) -> &linearf::AsyncRt { &self.rt }
}
