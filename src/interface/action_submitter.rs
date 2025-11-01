pub trait IActionSubmitter<A>: Send + Sync
where
    A: Send + Sync + Clone + 'static,
{
    fn submit(&self, action: A);
}
