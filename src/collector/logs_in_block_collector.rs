use std::sync::Arc;

use alloy::{
    primitives::BlockHash,
    providers::Provider,
    rpc::types::{
        Header,
        eth::{Filter, Log},
    },
};
use async_trait::async_trait;
use futures::StreamExt;

use crate::{CollectorStream, ICollector};

pub struct LogsInBlockCollector {
    provider: Arc<dyn Provider>,
    filter: Filter,
}

impl LogsInBlockCollector {
    pub fn new(provider: Arc<dyn Provider>, filter: Filter) -> Self {
        Self { provider, filter }
    }

    async fn block_to_logs(&self, block_hash: BlockHash) -> Option<Vec<Log>> {
        let logs = self
            .provider
            .get_logs(&self.filter.clone().at_block_hash(block_hash))
            .await;

        match logs {
            Ok(logs) => Some(logs),
            Err(e) => {
                tracing::error!(?block_hash, "fail to get logs: {e:#}");
                None
            }
        }
    }
}

#[async_trait]
impl ICollector<(Header, Vec<Log>)> for LogsInBlockCollector {
    fn name(&self) -> &str {
        "Logs In Block Collector"
    }

    async fn get_event_stream(&self) -> eyre::Result<CollectorStream<'_, (Header, Vec<Log>)>> {
        let mut stream = self.provider.subscribe_blocks().await?.into_stream();

        let stream = async_stream::stream! {
            while let Some(block) = stream.next().await {
                let logs = match self.block_to_logs(block.hash).await {
                    Some(logs) => logs,
                    None => continue,
                };

                yield (block, logs);
            }
        };

        Ok(Box::pin(stream))
    }
}
