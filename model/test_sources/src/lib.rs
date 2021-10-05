pub mod source {
    use linearf::{
        source::*, stream::Stream, AsyncRt, Item, Linearf, MaybeUtf8, New, Shared, State, Vars
    };
    use std::{
        pin::Pin,
        sync::{Arc, Weak}
    };

    pub struct Simple<L> {
        _linearf: Weak<L>
    }

    impl<L, R> New<L, R> for Simple<L>
    where
        L: linearf::Linearf<R> + Send + Sync,
        R: linearf::Registry
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

    impl<L, R> SimpleGenerator<L, R, BlankParams> for Simple<L>
    where
        L: linearf::Linearf<R> + Send + Sync,
        R: linearf::Registry
    {
        fn stream(
            &self,
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> Pin<Box<dyn Stream<Item = Item>>> {
            let s = linearf::stream::unfold(0..1000000, |mut it| async {
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
            _ctx: ReusableContext,
            _prev: (&Arc<Vars>, &Arc<Self::Params>),
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> bool {
            true
        }
    }
}

pub mod matcher {
    use linearf::{matcher::*, session::Vars, AsyncRt, Item, Linearf, New, Shared, State};
    use std::sync::{Arc, Weak};

    pub struct Substring<L> {
        _linearf: Weak<L>
    }

    impl<L, R> New<L, R> for Substring<L>
    where
        L: linearf::Linearf<R> + Send + Sync,
        R: linearf::Registry
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

    impl<L, R> SimpleScorer<L, R, BlankParams> for Substring<L>
    where
        L: linearf::Linearf<R> + Send + Sync,
        R: linearf::Registry
    {
        fn score(&self, (vars, _): (Arc<Vars>, Arc<Self::Params>), item: &Arc<Item>) -> Score {
            return if item.view_for_matcing().find(&vars.query).is_some() {
                Score::new(item.id, vec![1])
            } else {
                Score::new(item.id, vec![0])
            };
        }

        fn reusable(
            &self,
            _ctx: ReusableContext,
            (prev, _): (&Arc<Vars>, &Arc<Self::Params>),
            (senario, _): (&Arc<Vars>, &Arc<Self::Params>)
        ) -> bool {
            prev.query == senario.query
        }
    }
}

pub mod converter {
    use linearf::{converter::*, session::Vars, AsyncRt, Item, Linearf, New, Shared, State};
    use std::sync::Weak;

    struct OddEven {}

    impl<L, R> New<L, R> for OddEven
    where
        L: linearf::Linearf<R> + Send + Sync,
        R: linearf::Registry
    {
        fn new(_linearf: Weak<L>) -> Self
        where
            Self: Sized
        {
            Self {}
        }
    }

    impl<L, R> SimpleConverter<L, R> for OddEven
    where
        L: linearf::Linearf<R> + Send + Sync,
        R: linearf::Registry
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
