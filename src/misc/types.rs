use futures::Stream;
use std::pin::Pin;

use crate::interface::{collector::Collector, executor::Executor};

pub type CollectorStream<'a, E> = Pin<Box<dyn Stream<Item = E> + Send + 'a>>;

pub trait ActionSubmitter<A>: Send + Sync
where
    A: Send + Sync + Clone + 'static,
{
    fn submit(&self, action: A);
}

pub struct CollectorMap<E, F> {
    inner: Box<dyn Collector<E>>,
    f: F,
}

impl<E, F> CollectorMap<E, F> {
    pub fn new(collector: Box<dyn Collector<E>>, f: F) -> Self {
        Self { inner: collector, f }
    }
}

pub struct CollectorFilterMap<E, F> {
    inner: Box<dyn Collector<E>>,
    f: F,
}

impl<E, F> CollectorFilterMap<E, F> {
    pub fn new(collector: Box<dyn Collector<E>>, f: F) -> Self {
        Self { inner: collector, f }
    }
}

pub struct ExecutorMap<A, F> {
    inner: Box<dyn Executor<A>>,
    f: F,
}

impl<A, F> ExecutorMap<A, F> {
    pub fn new(executor: Box<dyn Executor<A>>, f: F) -> Self {
        Self { inner: executor, f }
    }
}
