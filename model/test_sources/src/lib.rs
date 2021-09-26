pub mod source {
    use linearf::{
        async_trait,
        source::{BlankParams, *},
        Item, MaybeUtf8, New, Shared, State, Vars
    };
    use std::sync::Arc;

    pub struct Simple {
        _state: Shared<State>
    }

    impl New for Simple {
        fn new(_state: &Shared<State>) -> Self
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

    #[async_trait]
    impl SimpleGenerator<BlankParams> for Simple {
        async fn generate(&self, tx: Transmitter, _senario: (&Arc<Vars>, &Arc<Self::Params>)) {
            for i in 0..1000 {
                tx.chunk(
                    (0..1000)
                        .map(|j| i + j + 1)
                        .map(|id| Item::new(id, "", MaybeUtf8::Utf8(i.to_string())))
                        .collect::<Vec<_>>()
                )
            }
        }

        async fn reusable(
            &self,
            _prev: (&Arc<Vars>, &Arc<Self::Params>),
            _senario: (&Arc<Vars>, &Arc<Self::Params>)
        ) -> bool {
            false
        }
    }
}

pub mod matcher {
    use linearf::{
        async_trait,
        matcher::{BlankParams, *},
        session::Vars,
        Item, New, Shared, State
    };
    use std::sync::Arc;

    pub struct Substring {
        _state: Shared<State>
    }

    impl New for Substring {
        fn new(_state: &Shared<State>) -> Self
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

    #[async_trait]
    impl SimpleScorer<BlankParams> for Substring {
        async fn score(
            &self,
            (vars, _): (&Arc<Vars>, &Arc<Self::Params>),
            item: &Arc<Item>
        ) -> Score {
            return if item.view_for_matcing().find(&vars.query).is_some() {
                Score::new(item.id, vec![1])
            } else {
                Score::new(item.id, vec![0])
            };
        }

        async fn reusable(
            &self,
            (prev, _): (&Arc<Vars>, &Arc<Self::Params>),
            (senario, _): (&Arc<Vars>, &Arc<Self::Params>)
        ) -> bool {
            prev.query == senario.query
        }
    }
}
