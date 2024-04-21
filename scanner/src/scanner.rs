use crate::db::PgStorage;
use crate::error::Result;
use crate::error::ScannerError;
use bitcoincore_rpc::RpcApi;
use crossbeam_channel::bounded;
use log::{error, info};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use crate::DEFAULT_BATCH_SZE;

pub struct Fetcher {
    pub retries: usize,
    pub client: bitcoincore_rpc::Client,
    pub storage: PgStorage,
    pub threads: usize,
}

impl Fetcher {
    pub async fn get_block_by_height(&self, height: u64) -> Result<()> {
        match self.client.get_block_hash(height) {
            Ok(block_id) => match self.client.get_block(&block_id) {
                Ok(block) => {
                    info!(
                        "height: {}, hash: {}, prev_hash: {}",
                        height, block_id, block.header.prev_blockhash
                    );

                    // TODO: parse header and save data to db.

                    Ok(())
                }
                Err(e) => Err(ScannerError::Custom(e.to_string())),
            },
            Err(e) => {
                if e.to_string().contains("out of range") {
                    Err(ScannerError::BlockNotFound(height))
                } else {
                    Err(ScannerError::Custom(e.to_string()))
                }
            }
        }
    }
}

pub struct Scanner {
    fetcher: Arc<Fetcher>,
}

impl Scanner {
    pub fn new(
        retries: usize,
        threads: usize,
        client: bitcoincore_rpc::Client,
        storage: PgStorage,
    ) -> Result<Self> {
        let fetcher = Fetcher {
            retries,
            client,
            storage,
            threads,
        };

        Ok(Self {
            fetcher: Arc::new(fetcher),
        })
    }

    pub async fn single_scan(&self, height: u64) -> Result<()> {
        info!("Syncing block: {}", height);

        self.fetcher.get_block_by_height(height).await?;
        self.fetcher.storage.upsert_tip(height as i64).await?;

        info!("Syncing block: {} complete", height);

        Ok(())
    }

    pub async fn range_scan(&self, start: u64, end: u64) -> Result<u64> {
        info!("Syncing [{},{}) ...", start, end);
        let concurrency = self.fetcher.threads; //how many spawned.
        let (sender, receiver) = bounded(concurrency);
        let last_height = Arc::new(AtomicU64::new(0));
        let succeed_cnt = Arc::new(AtomicU64::new(0));
        let caller_cloned = self.fetcher.clone();
        let last_height_cloned = last_height.clone();
        let succeed_cnt_cloned = succeed_cnt.clone();

        let producer_handle = tokio::task::spawn_blocking(move || {
            for h in start..end {
                let fut = task(
                    caller_cloned.clone(),
                    h,
                    last_height_cloned.clone(),
                    succeed_cnt_cloned.clone(),
                );

                sender.send(Some(fut)).unwrap();
            }

            for _ in 0..concurrency {
                sender.send(None).unwrap();
            }
        });

        let consumer_handles: Vec<_> = (0..concurrency)
            .map(move |_| {
                let r = receiver.clone();
                tokio::spawn(async move {
                    while let Ok(Some(fut)) = r.recv() {
                        fut.await;
                    }
                })
            })
            .collect();

        for h in consumer_handles {
            h.await?;
        }
        producer_handle.await?;

        info!("Syncing [{},{}) complete.", start, end);

        Ok(succeed_cnt.load(Ordering::Acquire))
    }

    pub async fn run(&self, start: u64, interval: Duration, single: bool) -> Result<()> {
        match single {
            true => {
                info!("Single syncing...");
                self.single_scan(start).await
            }
            false => {
                let mut height = start;
                let batch = (DEFAULT_BATCH_SZE * self.fetcher.threads) as u64;
                info!("Fast syncing...");
                loop {
                    let cnt = self.range_scan(height, height + batch).await?;
                    if cnt == batch {
                        height += batch;
                    } else {
                        break;
                    }
                }
                info!("Fast syncing complete.");
                loop {
                    if let Ok(h) = self.fetcher.storage.get_tip().await {
                        height = h as u64 + 1;
                    }

                    match self.fetcher.get_block_by_height(height).await {
                        Ok(_) => {
                            info!("Get block {} succeed", height);
                            self.fetcher.storage.upsert_tip(height as i64).await?;
                        }
                        Err(ScannerError::BlockNotFound(height)) => {
                            error!("Block {} not found", height)
                        }
                        Err(e) => {
                            error!("Get block {} error: {:?}", height, e);
                        }
                    }

                    tokio::time::sleep(interval).await;
                }
            }
        }
    }
}

async fn task(
    caller: Arc<Fetcher>,
    height: u64,
    last_height: Arc<AtomicU64>,
    succeed_cnt: Arc<AtomicU64>,
) {
    match caller.get_block_by_height(height).await {
        Ok(_) => {
            let h_old = last_height.load(Ordering::Acquire);
            if height > h_old {
                last_height.store(height, Ordering::Release);
                if let Err(e) = caller.storage.upsert_tip(height as i64).await {
                    error!("DB error: {:?}", e);
                }
            }
            succeed_cnt.fetch_add(1, Ordering::Release);
        }
        Err(ScannerError::BlockNotFound(h)) => error!("Block not found: {}", h),
        Err(e) => error!("Get block {} failed: {:?}", height, e),
    }
}
