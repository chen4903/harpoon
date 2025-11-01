use async_trait::async_trait;
use eyre::Result;
use futures::StreamExt;

use crate::interface::{ICollector, IExecutor, collector::CollectorStream};

pub struct CollectorMap<E, F> {
    inner: Box<dyn ICollector<E>>,
    f: F,
}

impl<E, F> CollectorMap<E, F> {
    pub fn new(collector: Box<dyn ICollector<E>>, f: F) -> Self {
        Self { inner: collector, f }
    }
}

#[async_trait]
impl<E1, E2, F> ICollector<E2> for CollectorMap<E1, F>
where
    E1: Send + Sync + 'static,
    E2: Send + Sync + 'static,
    F: Fn(E1) -> E2 + Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E2>> {
        let stream = self.inner.get_event_stream().await?;
        let f = self.f.clone();
        let stream = stream.map(f);
        Ok(Box::pin(stream))
    }
}

pub struct CollectorFilterMap<E, F> {
    inner: Box<dyn ICollector<E>>,
    f: F,
}

impl<E, F> CollectorFilterMap<E, F> {
    pub fn new(collector: Box<dyn ICollector<E>>, f: F) -> Self {
        Self { inner: collector, f }
    }
}

#[async_trait]
impl<E1, E2, F> ICollector<E2> for CollectorFilterMap<E1, F>
where
    E1: Send + Sync + 'static,
    E2: Send + Sync + 'static,
    F: Fn(E1) -> Option<E2> + Send + Sync + Clone + Copy + 'static,
{
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn get_event_stream(&self) -> Result<CollectorStream<'_, E2>> {
        let stream = self.inner.get_event_stream().await?;
        let f = self.f;
        let stream = stream.filter_map(move |v| async move { f(v) });
        Ok(Box::pin(stream))
    }
}

pub struct ExecutorMap<A, F> {
    inner: Box<dyn IExecutor<A>>,
    f: F,
}

impl<A, F> ExecutorMap<A, F> {
    pub fn new(executor: Box<dyn IExecutor<A>>, f: F) -> Self {
        Self { inner: executor, f }
    }
}

#[async_trait]
impl<A1, A2, F> IExecutor<A1> for ExecutorMap<A2, F>
where
    A1: Send + Sync + 'static,
    A2: Send + Sync + 'static,
    F: Fn(A1) -> Option<A2> + Send + Sync + Clone + 'static,
{
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn execute(&self, action: A1) -> Result<()> {
        let action = (self.f)(action);
        match action {
            Some(action) => self.inner.execute(action).await,
            None => Ok(()),
        }
    }
}
