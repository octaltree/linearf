pub use converter::Converter;
pub use matcher::Matcher;
pub use source::Source;
mod source {
    use linearf::source::*;
    use std::marker::PhantomData;
    pub struct Source<L> {
        phantom: PhantomData<L>,
        simple: linearf::source::Source<<test_sources::source::Simple<L> as IsSource>::Params>,
        osstr: linearf::source::Source<<test_sources::source::OsStr<L> as IsSource>::Params>
    }
    impl<L> Source<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        pub fn new(linearf: Weak<L>) -> Self
        where
            Self: Sized
        {
            Self {
                phantom: PhantomData,
                simple: <test_sources::source::Simple<L> as NewSource<L>>::new(linearf.clone()),
                osstr: <test_sources::source::OsStr<L> as NewSource<L>>::new(linearf.clone())
            }
        }
    }
    impl<L> SourceRegistry for Source<L>
    where
        L: linearf::Linearf + Send + Sync
    {
        fn parse<'de, D>(
            &self,
            name: &str,
            deserializer: D
        ) -> Option<Result<Arc<dyn Any + Send + Sync>, D::Error>>
        where
            D: serde::de::Deserializer<'de>
        {
            match name {
                "simple" => {
                    match <test_sources::source::Simple<L> as IsSource>::Params::deserialize(
                        deserializer
                    ) {
                        Ok(x) => Some(Ok(Arc::new(x))),
                        Err(e) => Some(Err(e))
                    }
                }
                "osstr" => match <test_sources::source::OsStr<L> as IsSource>::Params::deserialize(
                    deserializer
                ) {
                    Ok(x) => Some(Ok(Arc::new(x))),
                    Err(e) => Some(Err(e))
                },
                _ => None
            }
        }
        fn reusable(
            &self,
            name: &str,
            prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
            senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
        ) -> Reusable {
            match name {
                "simple" => match &self.simple {
                    linearf::source::Source::Simple(g) => {
                        let (prev_vars, prev_source) = prev;
                        let (senario_vars, senario_source) = senario;
                        if prev_source.is::<<test_sources::source::Simple<L> as IsSource>::Params>()
                            && senario_source
                                .is::<<test_sources::source::Simple<L> as IsSource>::Params>()
                        {
                            let prev_source: &Arc<
                                <test_sources::source::Simple<L> as IsSource>::Params
                            > = unsafe { std::mem::transmute(prev_source) };
                            let senario_source: &Arc<
                                <test_sources::source::Simple<L> as IsSource>::Params
                            > = unsafe { std::mem::transmute(senario_source) };
                            g.reusable((prev_vars, prev_source), (senario_vars, senario_source))
                        } else {
                            log::error!("mismatch source reusable params");
                            Reusable::None
                        }
                    }
                },
                "osstr" => match &self.osstr {
                    linearf::source::Source::Simple(g) => {
                        let (prev_vars, prev_source) = prev;
                        let (senario_vars, senario_source) = senario;
                        if prev_source.is::<<test_sources::source::OsStr<L> as IsSource>::Params>()
                            && senario_source
                                .is::<<test_sources::source::OsStr<L> as IsSource>::Params>()
                        {
                            let prev_source: &Arc<
                                <test_sources::source::OsStr<L> as IsSource>::Params
                            > = unsafe { std::mem::transmute(prev_source) };
                            let senario_source: &Arc<
                                <test_sources::source::OsStr<L> as IsSource>::Params
                            > = unsafe { std::mem::transmute(senario_source) };
                            g.reusable((prev_vars, prev_source), (senario_vars, senario_source))
                        } else {
                            log::error!("mismatch source reusable params");
                            Reusable::None
                        }
                    }
                },
                _ => Reusable::None
            }
        }
        fn stream(
            &self,
            name: &str,
            senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
        ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>> {
            match name {
                "simple" => match self.simple.clone() {
                    linearf::source::Source::Simple(g) => {
                        let (senario_vars, senario_source) = senario;
                        if !senario_source
                            .is::<<test_sources::source::Simple<L> as IsSource>::Params>()
                        {
                            log::error!("mismatch source stream params");
                            return Box::pin(empty());
                        }
                        let senario_source: &Arc<
                            <test_sources::source::Simple<L> as IsSource>::Params
                        > = unsafe { std::mem::transmute(senario_source) };
                        g.stream((&senario_vars, &senario_source))
                    }
                },
                "osstr" => match self.osstr.clone() {
                    linearf::source::Source::Simple(g) => {
                        let (senario_vars, senario_source) = senario;
                        if !senario_source
                            .is::<<test_sources::source::OsStr<L> as IsSource>::Params>()
                        {
                            log::error!("mismatch source stream params");
                            return Box::pin(empty());
                        }
                        let senario_source: &Arc<
                            <test_sources::source::OsStr<L> as IsSource>::Params
                        > = unsafe { std::mem::transmute(senario_source) };
                        g.stream((&senario_vars, &senario_source))
                    }
                },
                _ => Box::pin(empty())
            }
        }
    }
}
mod matcher {
    use linearf::matcher::*;
    use std::marker::PhantomData;
    pub struct Matcher<L> {
        phantom: PhantomData<L>,
        substring:
            linearf::matcher::Matcher<<test_sources::matcher::Substring<L> as IsMatcher>::Params>
    }
    impl<L> Matcher<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        pub fn new(linearf: Weak<L>) -> Self {
            Self {
                phantom: PhantomData,
                substring: <test_sources::matcher::Substring<L> as NewMatcher<L>>::new(
                    linearf.clone()
                )
            }
        }
    }
    impl<L> MatcherRegistry for Matcher<L>
    where
        L: linearf::Linearf + Send + Sync
    {
        fn parse<'de, D>(
            &self,
            name: &str,
            deserializer: D
        ) -> Option<Result<Arc<dyn Any + Send + Sync>, D::Error>>
        where
            D: serde::de::Deserializer<'de>
        {
            match name {
                "substring" => {
                    match <test_sources::matcher::Substring<L> as IsMatcher>::Params::deserialize(
                        deserializer
                    ) {
                        Ok(x) => Some(Ok(Arc::new(x))),
                        Err(e) => Some(Err(e))
                    }
                }
                _ => None
            }
        }
        fn reusable(
            &self,
            name: &str,
            prev: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
            senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>)
        ) -> Reusable {
            match name {
                "substring" => {
                    match &self.substring {
                        linearf::matcher::Matcher::Simple(s) => {
                            let (prev_vars, prev_matcher) = prev;
                            let (senario_vars, senario_matcher) = senario;
                            if prev_matcher . is :: < < test_sources :: matcher :: Substring < L > as IsMatcher > :: Params > () && senario_matcher . is :: < < test_sources :: matcher :: Substring < L > as IsMatcher > :: Params > () { let prev_matcher : & Arc < < test_sources :: matcher :: Substring < L > as IsMatcher > :: Params > = unsafe { std :: mem :: transmute (prev_matcher) } ; let senario_matcher : & Arc < < test_sources :: matcher :: Substring < L > as IsMatcher > :: Params > = unsafe { std :: mem :: transmute (senario_matcher) } ; s . reusable ((prev_vars , prev_matcher) , (senario_vars , senario_matcher)) } else { log :: error ! ("mismatch matcher reusable params") ; Reusable :: None }
                        }
                    }
                }
                _ => Reusable::None
            }
        }
        fn score(
            &self,
            name: &str,
            senario: (&Arc<Vars>, &Arc<dyn Any + Send + Sync>),
            items: impl Stream<Item = Arc<Item>> + Send + Sync + 'static
        ) -> Pin<Box<dyn Stream<Item = WithScore> + Send + Sync>> {
            match name {
                "substring" => match &self.substring {
                    linearf::matcher::Matcher::Simple(s) => {
                        let (senario_vars, senario_matcher) = senario;
                        if senario_matcher
                            .is::<<test_sources::matcher::Substring<L> as IsMatcher>::Params>()
                        {
                            let senario_matcher: &Arc<
                                <test_sources::matcher::Substring<L> as IsMatcher>::Params
                            > = unsafe { std::mem::transmute(senario_matcher) };
                            let s = s.clone();
                            let senario_vars = senario_vars.clone();
                            let senario_matcher = senario_matcher.clone();
                            Box::pin(items.map(move |x| {
                                let score = s.score((&senario_vars, &senario_matcher), &x);
                                (x, Arc::new(score))
                            }))
                        } else {
                            log::error!("mismatch matcher score params");
                            Box::pin(empty())
                        }
                    }
                },
                _ => Box::pin(empty())
            }
        }
    }
}
mod converter {
    use linearf::converter::*;
    use std::{collections::HashMap, marker::PhantomData};
    pub struct Converter<L> {
        phantom: PhantomData<L>,
        OddEven: linearf::converter::Converter<
            <test_sources::converter::OddEven<L> as IsConverter>::Params
        >
    }
    impl<L> Converter<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        pub fn new(linearf: Weak<L>) -> Self {
            Self {
                phantom: PhantomData,
                OddEven: <test_sources::converter::OddEven<L> as NewConverter<L>>::new(
                    linearf.clone()
                )
            }
        }
    }
    impl<L> ConverterRegistry for Converter<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        fn map_convert(
            &self,
            names: &[SmartString],
            items: impl Stream<Item = Item> + Send + Sync + 'static
        ) -> Result<Pin<Box<dyn Stream<Item = Item> + Send + Sync>>, MapConvertError> {
            let fs = names
                .iter()
                .map(|n| -> &str { &n })
                .map(
                    |name| -> Result<Box<dyn Fn(Item) -> Item + Send + Sync>, MapConvertError> {
                        match name {
                            "OddEven" => match &self.OddEven {
                                linearf::converter::Converter::Simple(c) => {
                                    let c = Arc::clone(c);
                                    Ok(Box::new(move |item| c.convert(item)))
                                }
                                linearf::converter::Converter::Reserve(_) => {
                                    Err(MapConvertError::ConverterNotFound(SmartString::from(name)))
                                }
                            },
                            _ => Err(MapConvertError::ConverterNotFound(SmartString::from(name)))
                        }
                    }
                )
                .collect::<Result<Vec<_>, _>>()?;
            let f = move |mut item: Item| -> Item {
                for f in &fs {
                    item = f(item);
                }
                item
            };
            Ok(Box::pin(items.map(f)))
        }
    }
}
