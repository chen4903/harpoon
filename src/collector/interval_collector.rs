use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::{CollectorStream, ICollector};

pub struct IntervalCollector {
    interval: Duration,
}

impl IntervalCollector {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }
}

#[async_trait]
impl ICollector<Instant> for IntervalCollector {
    fn name(&self) -> &str {
        "Interval Collector"
    }

    async fn get_event_stream(&self) -> eyre::Result<CollectorStream<'_, Instant>> {
        let stream = async_stream::stream! {
            loop {
                tokio::time::sleep(self.interval).await;
                yield Instant::now();
            }
        };

        Ok(Box::pin(stream))
    }
}
