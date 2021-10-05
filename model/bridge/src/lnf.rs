use linearf::{
    converter::ConverterRegistry, matcher::MatcherRegistry, source::SourceRegistry, AsyncRt,
    Linearf, Shared, State
};
use std::sync::{Arc, Weak};

pub struct Lnf {
    state: Shared<State>,
    registry: registry::Registry,
    rt: AsyncRt
}

impl Lnf {
    pub fn new(state: Shared<State>, rt: AsyncRt) -> Arc<Lnf> {
        Arc::new_cyclic(move |me| -> Lnf {
            let source = registry::Source::new(me.clone());
            let matcher = registry::Matcher::new(me.clone());
            let converter = registry::Converter::new(me.clone());
            let registry = registry::Registry {
                source,
                matcher,
                converter
            };
            Lnf {
                state,
                registry,
                rt
            }
        })
    }
}

impl Linearf<registry::Registry> for Lnf {
    fn state(&self) -> &Shared<State> { &self.state }

    fn registry(&self) -> &registry::Registry { &self.registry }

    fn runtime(&self) -> &linearf::AsyncRt { &self.rt }
}
