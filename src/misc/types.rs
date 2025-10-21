use futures::Stream;
use std::pin::Pin;

use crate::interface::{collector::CollectorInterface, executor::ExecutorInterface};

pub struct CollectorMap<E, F> {
    inner: Box<dyn CollectorInterface<E>>,
    f: F,
}

impl<E, F> CollectorMap<E, F> {
    pub fn new(collector: Box<dyn CollectorInterface<E>>, f: F) -> Self {
        Self { inner: collector, f }
    }
}

pub struct CollectorFilterMap<E, F> {
    inner: Box<dyn CollectorInterface<E>>,
    f: F,
}

impl<E, F> CollectorFilterMap<E, F> {
    pub fn new(collector: Box<dyn CollectorInterface<E>>, f: F) -> Self {
        Self { inner: collector, f }
    }
}

pub struct ExecutorMap<A, F> {
    inner: Box<dyn ExecutorInterface<A>>,
    f: F,
}

impl<A, F> ExecutorMap<A, F> {
    pub fn new(executor: Box<dyn ExecutorInterface<A>>, f: F) -> Self {
        Self { inner: executor, f }
    }
}
