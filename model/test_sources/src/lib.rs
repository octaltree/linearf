pub mod source {
    use linearf::{item::*, source::*};

    pub struct Simple<L> {
        _linearf: Weak<L>
    }

    impl<L> IsSource for Simple<L> {
        type Params = BlankParams;
    }

    impl<L> NewSource<L> for Simple<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        fn new(_linearf: Weak<L>) -> Source<<Self as IsSource>::Params>
        where
            Self: Sized
        {
            Source::from_simple(Self { _linearf })
        }
    }

    impl<L> SimpleGenerator for Simple<L> {
        fn stream(
            &self,
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>> {
            let s = futures::stream::unfold(0..1000000, |mut it| async {
                it.next().map(|i| {
                    let id = i + 1;
                    let item = Item::new(id, MaybeUtf8::Utf8(i.to_string()));
                    (item, it)
                })
            });
            Box::pin(s)
        }

        fn reusable(
            &self,
            _prev: (&Arc<Vars>, &Arc<Self::Params>),
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Reusable {
            Reusable::Same
        }
    }

    pub struct OsStr<L> {
        _linearf: Weak<L>
    }

    impl<L> IsSource for OsStr<L> {
        type Params = BlankParams;
    }

    impl<L> NewSource<L> for OsStr<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        fn new(_linearf: Weak<L>) -> Source<<Self as IsSource>::Params>
        where
            Self: Sized
        {
            Source::from_simple(Self { _linearf })
        }
    }

    impl<L> SimpleGenerator for OsStr<L> {
        fn stream(
            &self,
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>> {
            let d = match std::fs::read_dir("/home") {
                Ok(d) => d,
                Err(_) => return Box::pin(empty())
            };
            let it = d
                .filter_map(|e| Some(e.ok()?.path()))
                .enumerate()
                .filter_map(|(i, p)| Some((i.try_into().ok()?, p)))
                .map(|(id, p)| Item::new(id, MaybeUtf8::Os(p.into_os_string())));
            let s = futures::stream::unfold(it, |mut it| async { it.next().map(|i| (i, it)) });
            Box::pin(s)
        }

        fn reusable(
            &self,
            _prev: (&Arc<Vars>, &Arc<Self::Params>),
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Reusable {
            Reusable::Same
        }
    }
}

pub mod matcher {
    use linearf::matcher::*;

    pub struct Substring<L> {
        _linearf: Weak<L>
    }

    impl<L> IsMatcher for Substring<L> {
        type Params = BlankParams;
    }

    impl<L> NewMatcher<L> for Substring<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        fn new(_linearf: Weak<L>) -> Matcher<<Self as IsMatcher>::Params>
        where
            Self: Sized
        {
            Matcher::from_simple(Self { _linearf })
        }
    }

    impl<L> SimpleScorer for Substring<L> {
        fn score(&self, (vars, _): (&Arc<Vars>, &Arc<Self::Params>), item: &Arc<Item>) -> Score {
            return if item.view_for_matcing().find(&vars.query).is_some() {
                Score::value(item.id, 1)
            } else {
                Score::new_excluded()
            };
        }

        fn reusable(
            &self,
            (prev, _): (&Arc<Vars>, &Arc<Self::Params>),
            (senario, _): (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Reusable {
            if prev.query == senario.query {
                Reusable::Same
            } else {
                Reusable::None
            }
        }
    }
}

pub mod converter {
    use linearf::converter::*;
    use std::marker::PhantomData;

    pub struct OddEven<L> {
        phantom: PhantomData<L>
    }

    impl<L> IsConverter for OddEven<L> {
        type Params = Void;
    }

    impl<L> NewConverter<L> for OddEven<L>
    where
        L: linearf::Linearf + Send + Sync + 'static
    {
        fn new(_linearf: Weak<L>) -> Converter<<Self as IsConverter>::Params>
        where
            Self: Sized
        {
            Converter::from_simple(Self {
                phantom: PhantomData
            })
        }
    }

    impl<L> SimpleConverter for OddEven<L> {
        fn convert(&self, item: Item) -> Item {
            if let Ok(x) = item.view().parse::<i32>() {
                let view = Some(if x % 2 == 0 {
                    format!("e{}", x)
                } else {
                    format!("o{}", x)
                });
                Item { view, ..item }
            } else {
                item
            }
        }
    }
}
