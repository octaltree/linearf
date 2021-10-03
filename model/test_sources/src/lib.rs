pub mod source {
    use linearf::{source::*, stream::Stream, AsyncRt, Item, MaybeUtf8, New, Shared, State, Vars};
    use std::{pin::Pin, sync::Arc};

    pub struct Simple {
        _state: Shared<State>
    }

    impl New for Simple {
        fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
        where
            Self: Sized
        {
            Self {
                _state: _state.clone()
            }
        }
    }

    impl IsSource for Simple {
        type Params = BlankParams;
    }

    impl SimpleGenerator<BlankParams> for Simple {
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
            _ctx: ReusableContext<'_>,
            _prev: (&Arc<Vars>, &Arc<Self::Params>),
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> bool {
            true
        }
    }
}

pub mod matcher {
    use linearf::{matcher::*, session::Vars, AsyncRt, Item, New, Shared, State};
    use std::sync::Arc;

    pub struct Substring {
        _state: Shared<State>
    }

    impl New for Substring {
        fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
        where
            Self: Sized
        {
            Self {
                _state: _state.clone()
            }
        }
    }

    impl IsMatcher for Substring {
        type Params = BlankParams;
    }

    impl SimpleScorer<BlankParams> for Substring {
        fn score(&self, (vars, _): (Arc<Vars>, Arc<Self::Params>), item: &Arc<Item>) -> Score {
            return if item.view_for_matcing().find(&vars.query).is_some() {
                Score::new(item.id, vec![1])
            } else {
                Score::new(item.id, vec![0])
            };
        }

        fn reusable(
            &self,
            _ctx: ReusableContext<'_>,
            (prev, _): (&Arc<Vars>, &Arc<Self::Params>),
            (senario, _): (&Arc<Vars>, &Arc<Self::Params>)
        ) -> bool {
            prev.query == senario.query
        }
    }
}

pub mod converter {
    use linearf::{converter::*, session::Vars, AsyncRt, Item, New, Shared, State};

    struct OddEven {}

    impl New for OddEven {
        fn new(_state: &Shared<State>, _rt: &AsyncRt) -> Self
        where
            Self: Sized
        {
            Self {}
        }
    }

    impl SimpleConverter for OddEven {
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
