pub mod source {
    use linearf::{item::*, source::*};

    pub struct Simple<L> {
        _linearf: Weak<L>
    }

    impl<L> New<L> for Simple<L>
    where
        L: linearf::Linearf + Send + Sync
    {
        fn new(_linearf: Weak<L>) -> Self
        where
            Self: Sized
        {
            Self { _linearf }
        }
    }

    impl<L> IsSource for Simple<L> {
        type Params = BlankParams;
    }

    impl<L> SimpleGenerator<L, BlankParams> for Simple<L>
    where
        L: linearf::Linearf + Send + Sync
    {
        fn stream(
            &self,
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Pin<Box<dyn Stream<Item = Item> + Send + Sync>> {
            let s = futures::stream::unfold(0..1000000, |mut it| async {
                it.next().map(|i| {
                    let id = i + 1;
                    let item = Item::new(id, "number", MaybeUtf8::Utf8(i.to_string()));
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
}

pub mod matcher {
    use linearf::matcher::*;

    pub struct Substring<L> {
        _linearf: Weak<L>
    }

    impl<L> New<L> for Substring<L>
    where
        L: linearf::Linearf + Send + Sync
    {
        fn new(_linearf: Weak<L>) -> Self
        where
            Self: Sized
        {
            Self { _linearf }
        }
    }

    impl<L> IsMatcher for Substring<L> {
        type Params = BlankParams;
    }

    impl<L> SimpleScorer<L, BlankParams> for Substring<L>
    where
        L: linearf::Linearf + Send + Sync
    {
        fn score(&self, (vars, _): (&Arc<Vars>, &Arc<Self::Params>), item: &Arc<Item>) -> Score {
            return if item.view_for_matcing().find(&vars.query).is_some() {
                Score::new(item.id, vec![1])
            } else {
                Score::new(item.id, vec![0])
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

    struct OddEven {}

    impl<L> New<L> for OddEven
    where
        L: linearf::Linearf + Send + Sync
    {
        fn new(_linearf: Weak<L>) -> Self
        where
            Self: Sized
        {
            Self {}
        }
    }

    impl<L> SimpleConverter<L> for OddEven
    where
        L: linearf::Linearf + Send + Sync
    {
        fn convert(&self, item: Item) -> Item {
            if item.r#type != "number" {
                return item;
            }
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
