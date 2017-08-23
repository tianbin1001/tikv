// Copyright 2017 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use rocksdb::DB;
use std::sync::Arc;
use util::rocksdb::engine_metrics::*;
use std::thread::{Builder, JoinHandle};
use std::io;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

pub const DEFAULT_FLUSER_INTERVAL: u64 = 10000;

pub struct MetricsFlusher {
    engine: Arc<DB>,
    handle: Option<JoinHandle<()>>,
    sender: Option<Sender<bool>>,
    interval: Duration,
}

impl MetricsFlusher {
    pub fn new(engine: Arc<DB>, interval: Duration) -> MetricsFlusher {
        MetricsFlusher {
            engine: engine,
            handle: None,
            sender: None,
            interval: interval,
        }
    }

    pub fn start(&mut self) -> Result<(), io::Error> {
        let db = self.engine.clone();
        let (tx, rx) = mpsc::channel();
        let interval = self.interval;
        self.sender = Some(tx);
        let h = try!(
            Builder::new()
                .name(thd_name!("rocksb-metrics-flusher"))
                .spawn(move || {
                    while let Err(mpsc::RecvTimeoutError::Timeout) = rx.recv_timeout(interval) {
                        flush_metrics(&db);
                    }
                })
        );

        self.handle = Some(h);
        Ok(())
    }

    pub fn stop(&mut self) {
        let h = self.handle.take();
        if h.is_none() {
            return;
        }
        drop(self.sender.take().unwrap());
        if let Err(e) = h.unwrap().join() {
            error!("join metrics flusher failed {:?}", e);
            return;
        }
    }
}

fn flush_metrics(db: &DB) {
    for t in ENGINE_TICKER_TYPES_GAUGE {
        let v = db.get_statistics_ticker_count(*t);
        flush_engine_ticker_metrics(*t, v);
    }
    for t in ENGINE_TICKER_TYPES_COUNTER {
        let v = db.get_and_reset_statistics_ticker_count(*t);
        flush_engine_ticker_metrics(*t, v);
    }
    for t in ENGINE_HIST_TYPES {
        if let Some(v) = db.get_statistics_histogram(*t) {
            flush_engine_histogram_metrics(*t, v);
        }
    }
    flush_engine_properties(db);
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use std::time::Duration;
    use std::sync::Arc;
    use super::*;
    use rocksdb::{ColumnFamilyOptions, DBOptions};
    use util::rocksdb::{self, CFOptions};
    use storage::{CF_DEFAULT, CF_LOCK, CF_RAFT, CF_WRITE};
    use std::thread::sleep;

    #[test]
    fn test_metrics_flusher() {
        let path = TempDir::new("_test_metrics_flusher").unwrap();
        let db_opt = DBOptions::new();
        let cf_opts = ColumnFamilyOptions::new();
        let cfs_opts = vec![
            CFOptions::new(CF_DEFAULT, rocksdb::ColumnFamilyOptions::new()),
            CFOptions::new(CF_RAFT, rocksdb::ColumnFamilyOptions::new()),
            CFOptions::new(CF_LOCK, rocksdb::ColumnFamilyOptions::new()),
            CFOptions::new(CF_WRITE, cf_opts),
        ];
        let engine = Arc::new(
            rocksdb::new_engine_opt(path.path().to_str().unwrap(), db_opt, cfs_opts).unwrap(),
        );
        let mut metrics_flusher = MetricsFlusher::new(engine.clone(), Duration::from_millis(100));

        if let Err(e) = metrics_flusher.start() {
            error!("failed to start metrics flusher, error = {:?}", e);
        }

        let rtime = Duration::from_millis(300);
        sleep(rtime);

        metrics_flusher.stop();
    }
}
