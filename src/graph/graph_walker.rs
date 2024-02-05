use crate::{Id, Key, SchemaExt, SchemaResult, Typed, TypedGraph};
use std::iter::{once, Once};

#[derive(Clone)]
pub struct GraphWalker<'a, T, State, NK, EK, S, Front>
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
    State: Clone,
    Front: Iterator<Item = (State, SchemaResult<T, NK, EK, S>)>,
{
    g: &'a TypedGraph<NK, EK, S>,
    front: Front,
}

/// Type storing the value and state of the walker at a given point
pub struct WalkerTarget<T, State> {
    pub val: T,
    pub state: State,
}

impl<'a, T, State, NK, EK, S, Progress> GraphWalker<'a, T, State, NK, EK, S, Progress>
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
    State: Clone,
    Progress: Iterator<Item = (State, SchemaResult<T, NK, EK, S>)> + 'a,
{
    pub fn new(
        g: &'a TypedGraph<NK, EK, S>,
    ) -> GraphWalker<'a, (), (), NK, EK, S, impl Iterator<Item = ((), SchemaResult<(), NK, EK, S>)>>
    {
        GraphWalker {
            g,
            front: once(((), Ok(()))),
        }
    }

    pub fn new_from(
        g: &'a TypedGraph<NK, EK, S>,
        start: NK,
    ) -> GraphWalker<
        'a,
        &'a S::N,
        (),
        NK,
        EK,
        S,
        impl Iterator<Item = ((), SchemaResult<&'a S::N, NK, EK, S>)>,
    > {
        GraphWalker {
            g,
            front: g.get_node_safe(start).map(|n| ((), Ok(n))).into_iter(),
        }
    }

    pub fn set_state<NewState>(
        self,
        new_state: NewState,
    ) -> GraphWalker<
        'a,
        T,
        NewState,
        NK,
        EK,
        S,
        impl Iterator<Item = (NewState, SchemaResult<T, NK, EK, S>)>,
    >
    where
        NewState: Clone,
    {
        GraphWalker {
            g: self.g,
            front: self.front.map(move |(_, res)| (new_state.clone(), res)),
        }
    }

    /// Moves the walker forward without changing the state of the branch
    pub fn progress<'b, NewT, NextStep, StateAddition, WalkerStep>(
        self,
        walker_step: WalkerStep,
    ) -> GraphWalker<
        'b,
        NewT,
        State,
        NK,
        EK,
        S,
        impl Iterator<Item = (State, SchemaResult<NewT, NK, EK, S>)> + 'b,
    >
    where
        'a: 'b,
        'b: 'a,
        NewT: 'b,
        StateAddition: Clone + 'b,
        <NextStep as IntoIterator>::IntoIter: 'b,
        NextStep: IntoIterator<Item = (StateAddition, NewT)>,
        WalkerStep: Fn(T, &'a TypedGraph<NK, EK, S>) -> SchemaResult<NextStep, NK, EK, S> + 'b,
    {
        GraphWalker {
            g: self.g,
            front: self.front
                .map(move |(state, res)|
                    res.map_or_else::<Box<dyn Iterator<Item = SchemaResult<(StateAddition, NewT), NK, EK, S>>>, _, _>(
                            |e| {
                                Box::new(once(Err(e)))
                            },
                            |t| {
                                walker_step(t, self.g)
                                    .map_or_else::<Box<dyn Iterator<Item = SchemaResult<(StateAddition, NewT), NK, EK, S>>>, _, _>(
                                        |e| Box::new(once(Err(e))),
                                        |inner| Box::new(inner.into_iter().map(Ok))
                                    )
                            }
                        )
                        .map(move |res|
                            res.map_or_else(
                                |e| (state.clone(), Err(e)),
                                |(_, t)| (state.clone(), Ok(t))
                            )
                        )
                )
                .flatten()
        }
    }

    /// Moves the walker forward and adds more data to the state of the branch
    pub fn progress_with_state<'b, NewT, NextStep, StateAddition, WalkerStep, UpdateState>(
        self,
        walker_step: WalkerStep,
        update_state: UpdateState,
    ) -> GraphWalker<
        'b,
        NewT,
        State,
        NK,
        EK,
        S,
        impl Iterator<Item = (State, SchemaResult<NewT, NK, EK, S>)> + 'b,
    >
    where
        'a: 'b,
        'b: 'a,
        NewT: 'b,
        StateAddition: Clone + 'b,
        <NextStep as IntoIterator>::IntoIter: 'b,
        NextStep: IntoIterator<Item = (StateAddition, NewT)>,
        WalkerStep: Fn(T, &'a TypedGraph<NK, EK, S>) -> SchemaResult<NextStep, NK, EK, S> + 'b,
        UpdateState: Fn(State, StateAddition) -> State + 'b + Copy,
    {
        GraphWalker {
            g: self.g,
            front: self.front
                .map(move |(state, res)|
                    res.map_or_else::<Box<dyn Iterator<Item = SchemaResult<(StateAddition, NewT), NK, EK, S>>>, _, _>(
                        |e| {
                            Box::new(once(Err(e)))
                        },
                        |t| {
                            walker_step(t, self.g)
                                .map_or_else::<Box<dyn Iterator<Item = SchemaResult<(StateAddition, NewT), NK, EK, S>>>, _, _>(
                                    |e| Box::new(once(Err(e))),
                                    |inner| Box::new(inner.into_iter().map(Ok))
                                )
                        }
                    )
                        .map(move |res|
                            res.map_or_else(
                                |e| (state.clone(), Err(e)),
                                |(new_state, t)| (update_state(state.clone(), new_state), Ok(t))
                            )
                        )
                )
                .flatten()
        }
    }

    pub fn one(mut self) -> SchemaResult<Option<T>, NK, EK, S> {
        self.front.next().map_or_else(
            || Ok(None),
            |(_state, res)| res.map_or_else(|e| Err(e), |t| Ok(Some(t))),
        )
    }

    pub fn one_with_state(mut self) -> SchemaResult<Option<WalkerTarget<T, State>>, NK, EK, S> {
        self.front.next().map_or_else(
            || Ok(None),
            |(state, res)| {
                res.map_or_else(|e| Err(e), |t| Ok(Some(WalkerTarget { val: t, state })))
            },
        )
    }

    pub fn many<TCollection>(self) -> SchemaResult<TCollection, NK, EK, S>
    where
        TCollection: FromIterator<T>,
    {
        let mut results = Vec::new();
        for (_, res) in self.front {
            results.push(res?)
        }

        Ok(results.into_iter().collect())
    }

    pub fn many_with_state<TStateCollection>(self) -> SchemaResult<TStateCollection, NK, EK, S>
    where
        TStateCollection: FromIterator<WalkerTarget<T, State>>,
    {
        let mut results = Vec::new();
        for (state, res) in self.front {
            results.push(WalkerTarget { val: res?, state })
        }

        Ok(results.into_iter().collect())
    }
}

pub trait ToGraphWalker<NK, EK, S>: Id<NK>
where
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
{
    fn to_walker<'a>(
        &'a self,
        g: &'a TypedGraph<NK, EK, S>,
    ) -> SchemaResult<
        GraphWalker<'a, &Self, (), NK, EK, S, Once<((), SchemaResult<&'a Self, NK, EK, S>)>>,
        NK,
        EK,
        S,
    >;
}

impl<T, NK, EK, S> ToGraphWalker<NK, EK, S> for T
where
    T: Typed<Type = <S::N as Typed>::Type> + Id<NK>,
    NK: Key,
    EK: Key,
    S: SchemaExt<NK, EK>,
{
    fn to_walker<'a>(
        &'a self,
        g: &'a TypedGraph<NK, EK, S>,
    ) -> SchemaResult<
        GraphWalker<'a, &Self, (), NK, EK, S, Once<((), SchemaResult<&'a Self, NK, EK, S>)>>,
        NK,
        EK,
        S,
    > {
        // Make sure that the node has been inserted into the graph
        g.get_node(self.get_id())?;
        Ok(GraphWalker {
            g,
            front: once(((), Ok(self))),
        })
    }
}
