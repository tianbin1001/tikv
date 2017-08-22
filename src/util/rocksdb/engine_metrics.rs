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

use prometheus::{exponential_buckets, Counter, CounterVec, Gauge, GaugeVec, HistogramVec};
use rocksdb::{DBStatisticsHistogramType as HistType, DBStatisticsTickerType as TickerType,
              HistogramData, DB};
use storage::ALL_CFS;
use util::rocksdb;

pub const ROCKSDB_TOTAL_SST_FILES_SIZE: &'static str = "rocksdb.total-sst-files-size";
pub const ROCKSDB_TABLE_READERS_MEM: &'static str = "rocksdb.estimate-table-readers-mem";
pub const ROCKSDB_CUR_SIZE_ALL_MEM_TABLES: &'static str = "rocksdb.cur-size-all-mem-tables";
pub const ROCKSDB_ESTIMATE_NUM_KEYS: &'static str = "rocksdb.estimate-num-keys";
pub const ROCKSDB_PENDING_COMPACTION_BYTES: &'static str = "rocksdb.\
                                                            estimate-pending-compaction-bytes";
pub const ENGINE_TICKER_TYPES_GAUGE: &'static [TickerType] = &[
    TickerType::BlockCacheMiss,
    TickerType::BlockCacheHit,
    TickerType::BlockCacheIndexMiss,
    TickerType::BlockCacheIndexHit,
    TickerType::BlockCacheFilterMiss,
    TickerType::BlockCacheFilterHit,
    TickerType::BlockCacheDataMiss,
    TickerType::BlockCacheDataHit,
    TickerType::BlockCacheByteRead,
    TickerType::BlockCacheByteWrite,
    TickerType::BloomFilterUseful,
    TickerType::MemtableHit,
    TickerType::MemtableMiss,
    TickerType::GetHitL0,
    TickerType::GetHitL1,
    TickerType::GetHitL2AndUp,
    TickerType::NumberKeysWritten,
    TickerType::NumberKeysRead,
    TickerType::BytesWritten,
    TickerType::BytesRead,
    TickerType::IterBytesRead,
    TickerType::StallMicros,
    TickerType::NoIterators,
    TickerType::BloomFilterPrefixChecked,
    TickerType::BloomFilterPrefixUseful,
    TickerType::WalFileBytes,
    TickerType::CompactReadBytes,
    TickerType::CompactWriteBytes,
    TickerType::FlushWriteBytes,
];

pub const ENGINE_TICKER_TYPES_COUNTER: &'static [TickerType] = &[
    TickerType::CompactionKeyDropNewerEntry,
    TickerType::CompactionKeyDropObsolete,
    TickerType::CompactionKeyDropRangeDel,
    TickerType::CompactionRangeDelDropObsolete,
    TickerType::NumberDbSeek,
    TickerType::NumberDbNext,
    TickerType::NumberDbPrev,
    TickerType::NumberDbSeekFound,
    TickerType::NumberDbNextFound,
    TickerType::NumberDbPrevFound,
    TickerType::NoFileCloses,
    TickerType::NoFileOpens,
    TickerType::NoFileErrors,
    TickerType::WalFileSynced,
    TickerType::ReadAmpEstimateUsefulBytes,
    TickerType::ReadAmpTotalReadBytes,
];

pub const ENGINE_HIST_TYPES: &'static [HistType] = &[
    HistType::GetMicros,
    HistType::WriteMicros,
    HistType::CompactionTime,
    HistType::TableSyncMicros,
    HistType::CompactionOutfileSyncMicros,
    HistType::WalFileSyncMicros,
    HistType::ManifestFileSyncMicros,
    HistType::StallL0SlowdownCount,
    HistType::StallMemtableCompactionCount,
    HistType::StallL0NumFilesCount,
    HistType::HardRateLimitDelayCount,
    HistType::SoftRateLimitDelayCount,
    HistType::NumFilesInSingleCompaction,
    HistType::SeekMicros,
    HistType::WriteStall,
    HistType::SSTReadMicros,
    HistType::NumSubcompactionsScheduled,
    HistType::BytesPerRead,
    HistType::BytesPerWrite,
    HistType::BytesCompressed,
    HistType::BytesDecompressed,
    HistType::CompressionTimesNanos,
    HistType::DecompressionTimesNanos,
];

pub fn flush_engine_ticker_metrics(t: TickerType, value: u64) {
    match t {
        TickerType::BlockCacheMiss => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_miss"])
                .set(value as f64);
        }
        TickerType::BlockCacheHit => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_hit"])
                .set(value as f64);
        }
        TickerType::BlockCacheIndexMiss => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_index_miss"])
                .set(value as f64);
        }
        TickerType::BlockCacheIndexHit => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_index_hit"])
                .set(value as f64);
        }
        TickerType::BlockCacheFilterMiss => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_filter_miss"])
                .set(value as f64);
        }
        TickerType::BlockCacheFilterHit => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_filter_hit"])
                .set(value as f64);
        }
        TickerType::BlockCacheDataMiss => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_data_miss"])
                .set(value as f64);
        }
        TickerType::BlockCacheDataHit => {
            STORE_ENGINE_CACHE_EFFICIENCY_VEC
                .with_label_values(&["block_cache_data_hit"])
                .set(value as f64);
        }
        TickerType::BlockCacheByteRead => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["block_cache_byte_read"])
                .set(value as f64);
        }
        TickerType::BlockCacheByteWrite => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["block_cache_byte_write"])
                .set(value as f64);
        }
        TickerType::BloomFilterUseful => {
            STORE_ENGINE_BLOOM_EFFICIENCY_VEC
                .with_label_values(&["bloom_useful"])
                .set(value as f64);
        }
        TickerType::MemtableHit => {
            STORE_ENGINE_MEMTABLE_EFFICIENCY_VEC
                .with_label_values(&["memtable_hit"])
                .set(value as f64);
        }
        TickerType::MemtableMiss => {
            STORE_ENGINE_MEMTABLE_EFFICIENCY_VEC
                .with_label_values(&["memtable_miss"])
                .set(value as f64);
        }
        TickerType::GetHitL0 => {
            STORE_ENGINE_READ_SURVED_VEC
                .with_label_values(&["get_hit_l0"])
                .set(value as f64);
        }
        TickerType::GetHitL1 => {
            STORE_ENGINE_READ_SURVED_VEC
                .with_label_values(&["get_hit_l1"])
                .set(value as f64);
        }
        TickerType::GetHitL2AndUp => {
            STORE_ENGINE_READ_SURVED_VEC
                .with_label_values(&["get_hit_l2_and_up"])
                .set(value as f64);
        }
        TickerType::CompactionKeyDropNewerEntry => {
            STORE_ENGINE_COMPACTION_DROP_VEC
                .with_label_values(&["compaction_key_drop_newer_entry"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::CompactionKeyDropObsolete => {
            STORE_ENGINE_COMPACTION_DROP_VEC
                .with_label_values(&["compaction_key_drop_obsolete"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::CompactionKeyDropRangeDel => {
            STORE_ENGINE_COMPACTION_DROP_VEC
                .with_label_values(&["compaction_key_drop_range_del"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::CompactionRangeDelDropObsolete => {
            STORE_ENGINE_COMPACTION_DROP_VEC
                .with_label_values(&["range_del_drop_obsolete"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NumberKeysWritten => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["keys_written"])
                .set(value as f64);
        }
        TickerType::NumberKeysRead => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["keys_read"])
                .set(value as f64);
        }
        TickerType::BytesWritten => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["bytes_written"])
                .set(value as f64);
        }
        TickerType::BytesRead => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["bytes_read"])
                .set(value as f64);
        }
        TickerType::NumberDbSeek => {
            STORE_ENGINE_LOCATE_VEC
                .with_label_values(&["number_db_seek"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NumberDbNext => {
            STORE_ENGINE_LOCATE_VEC
                .with_label_values(&["number_db_next"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NumberDbPrev => {
            STORE_ENGINE_LOCATE_VEC
                .with_label_values(&["number_db_prev"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NumberDbSeekFound => {
            STORE_ENGINE_LOCATE_VEC
                .with_label_values(&["number_db_seek_found"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NumberDbNextFound => {
            STORE_ENGINE_LOCATE_VEC
                .with_label_values(&["number_db_next_found"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NumberDbPrevFound => {
            STORE_ENGINE_LOCATE_VEC
                .with_label_values(&["number_db_prev_found"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::IterBytesRead => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["iter_bytes_read"])
                .set(value as f64);
        }
        TickerType::NoFileCloses => {
            STORE_ENGINE_FILE_STATUS_VEC
                .with_label_values(&["no_file_closes"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NoFileOpens => {
            STORE_ENGINE_FILE_STATUS_VEC
                .with_label_values(&["no_file_opens"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::NoFileErrors => {
            STORE_ENGINE_FILE_STATUS_VEC
                .with_label_values(&["no_file_errors"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::StallMicros => {
            STORE_ENGINE_STALL_MICROS.set(value as f64);
        }
        TickerType::NoIterators => {
            STORE_ENGINE_NO_ITERATORS.set(value as f64);
        }
        TickerType::BloomFilterPrefixChecked => {
            STORE_ENGINE_BLOOM_EFFICIENCY_VEC
                .with_label_values(&["bloom_prefix_checked"])
                .set(value as f64);
        }
        TickerType::BloomFilterPrefixUseful => {
            STORE_ENGINE_BLOOM_EFFICIENCY_VEC
                .with_label_values(&["bloom_prefix_useful"])
                .set(value as f64);
        }
        TickerType::WalFileSynced => {
            STORE_ENGINE_WAL_FILE_SYNCED.inc_by(value as f64).unwrap();
        }
        TickerType::WalFileBytes => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["wal_file_bytes"])
                .set(value as f64);
        }
        TickerType::CompactReadBytes => {
            STORE_ENGINE_COMPACTION_FLOW_VEC
                .with_label_values(&["bytes_read"])
                .set(value as f64);
        }
        TickerType::CompactWriteBytes => {
            STORE_ENGINE_COMPACTION_FLOW_VEC
                .with_label_values(&["bytes_written"])
                .set(value as f64);
        }
        TickerType::FlushWriteBytes => {
            STORE_ENGINE_FLOW_VEC
                .with_label_values(&["flush_write_bytes"])
                .set(value as f64);
        }
        TickerType::ReadAmpEstimateUsefulBytes => {
            STORE_ENGINE_READ_AMP_FLOW_VEC
                .with_label_values(&["read_amp_estimate_useful_bytes"])
                .inc_by(value as f64)
                .unwrap();
        }
        TickerType::ReadAmpTotalReadBytes => {
            STORE_ENGINE_READ_AMP_FLOW_VEC
                .with_label_values(&["read_amp_total_read_bytes"])
                .inc_by(value as f64)
                .unwrap();
        }
    }
}

pub fn flush_engine_histogram_metrics(t: HistType, value: HistogramData) {
    match t {
        HistType::GetMicros => {
            STORE_ENGINE_GET_MICROS_VEC
                .with_label_values(&["get_median"])
                .set(value.median);
            STORE_ENGINE_GET_MICROS_VEC
                .with_label_values(&["get_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_GET_MICROS_VEC
                .with_label_values(&["get_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_GET_MICROS_VEC
                .with_label_values(&["get_average"])
                .set(value.average);
            STORE_ENGINE_GET_MICROS_VEC
                .with_label_values(&["get_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::WriteMicros => {
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["write_median"])
                .set(value.median);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["write_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["write_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["write_average"])
                .set(value.average);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["write_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::CompactionTime => {
            STORE_ENGINE_COMPACTION_TIME_VEC
                .with_label_values(&["compaction_time_median"])
                .set(value.median);
            STORE_ENGINE_COMPACTION_TIME_VEC
                .with_label_values(&["compaction_time_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_COMPACTION_TIME_VEC
                .with_label_values(&["compaction_time_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_COMPACTION_TIME_VEC
                .with_label_values(&["compaction_time_average"])
                .set(value.average);
            STORE_ENGINE_COMPACTION_TIME_VEC
                .with_label_values(&["compaction_time_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::TableSyncMicros => {
            STORE_ENGINE_TABLE_SYNC_MICROS_VEC
                .with_label_values(&["table_sync_median"])
                .set(value.median);
            STORE_ENGINE_TABLE_SYNC_MICROS_VEC
                .with_label_values(&["table_sync_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_TABLE_SYNC_MICROS_VEC
                .with_label_values(&["table_sync_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_TABLE_SYNC_MICROS_VEC
                .with_label_values(&["table_sync_average"])
                .set(value.average);
            STORE_ENGINE_TABLE_SYNC_MICROS_VEC
                .with_label_values(&["table_sync_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::CompactionOutfileSyncMicros => {
            STORE_ENGINE_COMPACTION_OUTFILE_SYNC_MICROS_VEC
                .with_label_values(&["compaction_outfile_sync_median"])
                .set(value.median);
            STORE_ENGINE_COMPACTION_OUTFILE_SYNC_MICROS_VEC
                .with_label_values(&["compaction_outfile_sync_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_COMPACTION_OUTFILE_SYNC_MICROS_VEC
                .with_label_values(&["compaction_outfile_sync_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_COMPACTION_OUTFILE_SYNC_MICROS_VEC
                .with_label_values(&["compaction_outfile_sync_average"])
                .set(value.average);
            STORE_ENGINE_COMPACTION_OUTFILE_SYNC_MICROS_VEC
                .with_label_values(&["compaction_outfile_sync_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::WalFileSyncMicros => {
            STORE_ENGINE_WAL_FILE_SYNC_MICROS_VEC
                .with_label_values(&["wal_file_sync_median"])
                .set(value.median);
            STORE_ENGINE_WAL_FILE_SYNC_MICROS_VEC
                .with_label_values(&["wal_file_sync_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_WAL_FILE_SYNC_MICROS_VEC
                .with_label_values(&["wal_file_sync_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_WAL_FILE_SYNC_MICROS_VEC
                .with_label_values(&["wal_file_sync_average"])
                .set(value.average);
            STORE_ENGINE_WAL_FILE_SYNC_MICROS_VEC
                .with_label_values(&["wal_file_sync_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::ManifestFileSyncMicros => {
            STORE_ENGINE_MANIFEST_FILE_SYNC_MICROS_VEC
                .with_label_values(&["manifest_file_sync_median"])
                .set(value.median);
            STORE_ENGINE_MANIFEST_FILE_SYNC_MICROS_VEC
                .with_label_values(&["manifest_file_sync_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_MANIFEST_FILE_SYNC_MICROS_VEC
                .with_label_values(&["manifest_file_sync_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_MANIFEST_FILE_SYNC_MICROS_VEC
                .with_label_values(&["manifest_file_sync_average"])
                .set(value.average);
            STORE_ENGINE_MANIFEST_FILE_SYNC_MICROS_VEC
                .with_label_values(&["manifest_file_sync_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::StallL0SlowdownCount => {
            STORE_ENGINE_STALL_L0_SLOWDOWN_COUNT_VEC
                .with_label_values(&["stall_l0_slowdown_count_median"])
                .set(value.median);
            STORE_ENGINE_STALL_L0_SLOWDOWN_COUNT_VEC
                .with_label_values(&["stall_l0_slowdown_count_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_STALL_L0_SLOWDOWN_COUNT_VEC
                .with_label_values(&["stall_l0_slowdown_count_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_STALL_L0_SLOWDOWN_COUNT_VEC
                .with_label_values(&["stall_l0_slowdown_count_average"])
                .set(value.average);
            STORE_ENGINE_STALL_L0_SLOWDOWN_COUNT_VEC
                .with_label_values(&["stall_l0_slowdown_count_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::StallMemtableCompactionCount => {
            STORE_ENGINE_STALL_MEMTABLE_COMPACTION_COUNT_VEC
                .with_label_values(&["stall_memtable_compaction_count_median"])
                .set(value.median);
            STORE_ENGINE_STALL_MEMTABLE_COMPACTION_COUNT_VEC
                .with_label_values(&["stall_memtable_compaction_count_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_STALL_MEMTABLE_COMPACTION_COUNT_VEC
                .with_label_values(&["stall_memtable_compaction_count_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_STALL_MEMTABLE_COMPACTION_COUNT_VEC
                .with_label_values(&["stall_memtable_compaction_count_average"])
                .set(value.average);
            STORE_ENGINE_STALL_MEMTABLE_COMPACTION_COUNT_VEC
                .with_label_values(&["stall_memtable_compaction_count_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::StallL0NumFilesCount => {
            STORE_ENGINE_STALL_LO_NUM_FILES_COUNT_VEC
                .with_label_values(&["stall_l0_num_files_count_median"])
                .set(value.median);
            STORE_ENGINE_STALL_LO_NUM_FILES_COUNT_VEC
                .with_label_values(&["stall_l0_num_files_count_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_STALL_LO_NUM_FILES_COUNT_VEC
                .with_label_values(&["stall_l0_num_files_count_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_STALL_LO_NUM_FILES_COUNT_VEC
                .with_label_values(&["stall_l0_num_files_count_average"])
                .set(value.average);
            STORE_ENGINE_STALL_LO_NUM_FILES_COUNT_VEC
                .with_label_values(&["stall_l0_num_files_count_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::HardRateLimitDelayCount => {
            STORE_ENGINE_HARD_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["hard_rate_limit_delay_median"])
                .set(value.median);
            STORE_ENGINE_HARD_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["hard_rate_limit_delay_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_HARD_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["hard_rate_limit_delay_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_HARD_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["hard_rate_limit_delay_average"])
                .set(value.average);
            STORE_ENGINE_HARD_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["hard_rate_limit_delay_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::SoftRateLimitDelayCount => {
            STORE_ENGINE_SOFT_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["soft_rate_limit_delay_median"])
                .set(value.median);
            STORE_ENGINE_SOFT_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["soft_rate_limit_delay_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_SOFT_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["soft_rate_limit_delay_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_SOFT_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["soft_rate_limit_delay_average"])
                .set(value.average);
            STORE_ENGINE_SOFT_RATE_LIMIT_DELAY_COUNT_VEC
                .with_label_values(&["soft_rate_limit_delay_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::NumFilesInSingleCompaction => {
            STORE_ENGINE_NUM_FILES_IN_SINGLE_COMPACTION_VEC
                .with_label_values(&["num_files_in_single_compaction_median"])
                .set(value.median);
            STORE_ENGINE_NUM_FILES_IN_SINGLE_COMPACTION_VEC
                .with_label_values(&["num_files_in_single_compaction_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_NUM_FILES_IN_SINGLE_COMPACTION_VEC
                .with_label_values(&["num_files_in_single_compaction_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_NUM_FILES_IN_SINGLE_COMPACTION_VEC
                .with_label_values(&["num_files_in_single_compaction_average"])
                .set(value.average);
            STORE_ENGINE_NUM_FILES_IN_SINGLE_COMPACTION_VEC
                .with_label_values(&["num_files_in_single_compaction_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::SeekMicros => {
            STORE_ENGINE_SEEK_MICROS_VEC
                .with_label_values(&["seek_median"])
                .set(value.median);
            STORE_ENGINE_SEEK_MICROS_VEC
                .with_label_values(&["seek_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_SEEK_MICROS_VEC
                .with_label_values(&["seek_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_SEEK_MICROS_VEC
                .with_label_values(&["seek_average"])
                .set(value.average);
            STORE_ENGINE_SEEK_MICROS_VEC
                .with_label_values(&["seek_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::WriteStall => {
            STORE_ENGINE_WRITE_STALL_VEC
                .with_label_values(&["write_stall_median"])
                .set(value.median);
            STORE_ENGINE_WRITE_STALL_VEC
                .with_label_values(&["write_stall_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_WRITE_STALL_VEC
                .with_label_values(&["write_stall_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_WRITE_STALL_VEC
                .with_label_values(&["write_stall_average"])
                .set(value.average);
            STORE_ENGINE_WRITE_STALL_VEC
                .with_label_values(&["write_stall_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::SSTReadMicros => {
            STORE_ENGINE_SST_READ_MICROS_VEC
                .with_label_values(&["sst_read_micros_median"])
                .set(value.median);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["sst_read_micros_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["sst_read_micros_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["sst_read_micros_average"])
                .set(value.average);
            STORE_ENGINE_WRITE_MICROS_VEC
                .with_label_values(&["sst_read_micros_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::NumSubcompactionsScheduled => {
            STORE_ENGINE_NUM_SUBCOMPACTION_SCHEDULED_VEC
                .with_label_values(&["num_subcompaction_scheduled_median"])
                .set(value.median);
            STORE_ENGINE_NUM_SUBCOMPACTION_SCHEDULED_VEC
                .with_label_values(&["num_subcompaction_scheduled_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_NUM_SUBCOMPACTION_SCHEDULED_VEC
                .with_label_values(&["num_subcompaction_scheduled_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_NUM_SUBCOMPACTION_SCHEDULED_VEC
                .with_label_values(&["num_subcompaction_scheduled_average"])
                .set(value.average);
            STORE_ENGINE_NUM_SUBCOMPACTION_SCHEDULED_VEC
                .with_label_values(&["num_subcompaction_scheduled_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::BytesPerRead => {
            STORE_ENGINE_BYTES_PER_READ_VEC
                .with_label_values(&["bytes_per_read_median"])
                .set(value.median);
            STORE_ENGINE_BYTES_PER_READ_VEC
                .with_label_values(&["bytes_per_read_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_BYTES_PER_READ_VEC
                .with_label_values(&["bytes_per_read_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_BYTES_PER_READ_VEC
                .with_label_values(&["bytes_per_read_average"])
                .set(value.average);
            STORE_ENGINE_BYTES_PER_READ_VEC
                .with_label_values(&["bytes_per_read_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::BytesPerWrite => {
            STORE_ENGINE_BYTES_PER_WRITE_VEC
                .with_label_values(&["bytes_per_write_median"])
                .set(value.median);
            STORE_ENGINE_BYTES_PER_WRITE_VEC
                .with_label_values(&["bytes_per_write_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_BYTES_PER_WRITE_VEC
                .with_label_values(&["bytes_per_write_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_BYTES_PER_WRITE_VEC
                .with_label_values(&["bytes_per_write_average"])
                .set(value.average);
            STORE_ENGINE_BYTES_PER_WRITE_VEC
                .with_label_values(&["bytes_per_write_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::BytesCompressed => {
            STORE_ENGINE_BYTES_COMPRESSED_VEC
                .with_label_values(&["bytes_compressed_median"])
                .set(value.median);
            STORE_ENGINE_BYTES_COMPRESSED_VEC
                .with_label_values(&["bytes_compressed_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_BYTES_COMPRESSED_VEC
                .with_label_values(&["bytes_compressed_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_BYTES_COMPRESSED_VEC
                .with_label_values(&["bytes_compressed_average"])
                .set(value.average);
            STORE_ENGINE_BYTES_COMPRESSED_VEC
                .with_label_values(&["bytes_compressed_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::BytesDecompressed => {
            STORE_ENGINE_BYTES_DECOMPRESSED_VEC
                .with_label_values(&["bytes_decompressed_median"])
                .set(value.median);
            STORE_ENGINE_BYTES_DECOMPRESSED_VEC
                .with_label_values(&["bytes_decompressed_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_BYTES_DECOMPRESSED_VEC
                .with_label_values(&["bytes_decompressed_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_BYTES_DECOMPRESSED_VEC
                .with_label_values(&["bytes_decompressed_average"])
                .set(value.average);
            STORE_ENGINE_BYTES_DECOMPRESSED_VEC
                .with_label_values(&["bytes_decompressed_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::CompressionTimesNanos => {
            STORE_ENGINE_COMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["compression_time_nanos_median"])
                .set(value.median);
            STORE_ENGINE_COMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["compression_time_nanos_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_COMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["compression_time_nanos_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_COMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["compression_time_nanos_average"])
                .set(value.average);
            STORE_ENGINE_COMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["compression_time_nanos_standard_deviation"])
                .set(value.standard_deviation);
        }
        HistType::DecompressionTimesNanos => {
            STORE_ENGINE_DECOMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["decompression_time_nanos_median"])
                .set(value.median);
            STORE_ENGINE_DECOMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["decompression_time_nanos_percentile95"])
                .set(value.percentile95);
            STORE_ENGINE_DECOMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["decompression_time_nanos_percentile99"])
                .set(value.percentile99);
            STORE_ENGINE_DECOMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["decompression_time_nanos_average"])
                .set(value.average);
            STORE_ENGINE_DECOMPRESSION_TIMES_NANOS_VEC
                .with_label_values(&["decompression_time_nanos_standard_deviation"])
                .set(value.standard_deviation);
        }
        _ => {}
    }
}

pub fn flush_engine_properties(engine: &DB) {
    for cf in ALL_CFS {
        let handle = rocksdb::get_cf_handle(engine, cf).unwrap();
        // It is important to monitor each cf's size, especially the "raft" and "lock" column
        // families.
        let cf_used_size = engine
            .get_property_int_cf(handle, ROCKSDB_TOTAL_SST_FILES_SIZE)
            .expect("rocksdb is too old, missing total-sst-files-size property");
        STORE_ENGINE_SIZE_GAUGE_VEC
            .with_label_values(&[cf])
            .set(cf_used_size as f64);

        // For block cache usage
        let block_cache_usage = engine.get_block_cache_usage_cf(handle);
        STORE_ENGINE_BLOCK_CACHE_USAGE_GAUGE_VEC
            .with_label_values(&[cf])
            .set(block_cache_usage as f64);

        // TODO: find a better place to record these metrics.
        // Refer: https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB
        // For index and filter blocks memory
        if let Some(readers_mem) = engine.get_property_int_cf(handle, ROCKSDB_TABLE_READERS_MEM) {
            STORE_ENGINE_MEMORY_GAUGE_VEC
                .with_label_values(&[cf, "readers-mem"])
                .set(readers_mem as f64);
        }

        // For memtable
        if let Some(mem_table) =
            engine.get_property_int_cf(handle, ROCKSDB_CUR_SIZE_ALL_MEM_TABLES)
        {
            STORE_ENGINE_MEMORY_GAUGE_VEC
                .with_label_values(&[cf, "mem-tables"])
                .set(mem_table as f64);
        }

        // TODO: add cache usage and pinned usage.

        if let Some(num_keys) = engine.get_property_int_cf(handle, ROCKSDB_ESTIMATE_NUM_KEYS) {
            STORE_ENGINE_ESTIMATE_NUM_KEYS_VEC
                .with_label_values(&[cf])
                .set(num_keys as f64);
        }

        // Pending compaction bytes
        if let Some(pending_compaction_bytes) =
            engine.get_property_int_cf(handle, ROCKSDB_PENDING_COMPACTION_BYTES)
        {
            STORE_ENGINE_PENDING_COMACTION_BYTES_VEC
                .with_label_values(&[cf])
                .set(pending_compaction_bytes as f64);
        }
    }
}

lazy_static!{
    pub static ref STORE_ENGINE_SIZE_GAUGE_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_size_bytes",
            "Sizes of each column families",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_BLOCK_CACHE_USAGE_GAUGE_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_block_cache_size_bytes",
            "Usage of each column families' block cache",
            &["cf"]
        ).unwrap();

    pub static ref STORE_ENGINE_MEMORY_GAUGE_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_memory_bytes",
            "Sizes of each column families",
            &["cf", "type"]
        ).unwrap();

    pub static ref STORE_ENGINE_ESTIMATE_NUM_KEYS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_estimate_num_keys",
            "Estimate num keys of each column families",
            &["cf"]
        ).unwrap();

    pub static ref STORE_ENGINE_CACHE_EFFICIENCY_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_cache_efficiency",
            "Efficiency of rocksdb's block cache",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_MEMTABLE_EFFICIENCY_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_memtable_efficiency",
            "Hit and miss of memtable",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_READ_SURVED_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_get_served",
            "Get queries served by",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_BLOOM_EFFICIENCY_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_bloom_efficiency",
            "Efficiency of rocksdb's bloom filter",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_FLOW_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_flow_bytes",
            "Bytes and keys of read/written",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_STALL_MICROS: Gauge =
        register_gauge!(
            "tikv_engine_stall_micro_seconds",
            "Stall micros"
        ).unwrap();

    pub static ref STORE_ENGINE_GET_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_get_micro_seconds",
            "Histogram of get micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_WRITE_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_write_micro_seconds",
            "Histogram of write micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_COMPACTION_TIME_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_compaction_time",
            "Histogram of compaction time",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_TABLE_SYNC_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_table_sync_micro_seconds",
            "Histogram of table sync micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_COMPACTION_OUTFILE_SYNC_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_compaction_outfile_sync_micro_seconds",
            "Histogram of compaction outfile sync micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_MANIFEST_FILE_SYNC_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_manifest_file_sync_micro_seconds",
            "Histogram of manifest file sync micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_WAL_FILE_SYNC_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_wal_file_sync_micro_seconds",
            "Histogram of WAL file sync micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_STALL_L0_SLOWDOWN_COUNT_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_stall_l0_slowdown_count",
            "Histogram of stall l0 slowdown count",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_STALL_MEMTABLE_COMPACTION_COUNT_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_stall_memtable_compaction_count",
            "Histogram of stall memtable compaction count",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_STALL_LO_NUM_FILES_COUNT_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_stall_l0_num_files_count",
            "Histogram of stall l0 num files count",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_HARD_RATE_LIMIT_DELAY_COUNT_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_hard_rate_limit_delay_count",
            "Histogram of hard rate limit delay count",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_SOFT_RATE_LIMIT_DELAY_COUNT_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_soft_rate_limit_delay_count",
            "Histogram of soft rate limit delay count",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_NUM_FILES_IN_SINGLE_COMPACTION_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_num_files_in_single_compaction",
            "Histogram of number of files in single compaction",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_SEEK_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_seek_micro_seconds",
            "Histogram of seek micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_WRITE_STALL_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_write_stall",
            "Histogram of write stall",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_SST_READ_MICROS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_sst_read_micros",
            "Histogram of SST read micros",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_NUM_SUBCOMPACTION_SCHEDULED_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_num_subcompaction_scheduled",
            "Histogram of number of subcompaction scheduled",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_BYTES_PER_READ_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_bytes_per_read",
            "Histogram of bytes per read",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_BYTES_PER_WRITE_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_bytes_per_write",
            "Histogram of bytes per write",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_BYTES_COMPRESSED_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_bytes_compressed",
            "Histogram of bytes compressed",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_BYTES_DECOMPRESSED_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_bytes_decompressed",
            "Histogram of bytes decompressed",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_COMPRESSION_TIMES_NANOS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_compression_time_nanos",
            "Histogram of compression time nanos",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_DECOMPRESSION_TIMES_NANOS_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_decompression_time_nanos",
            "Histogram of decompression time nanos",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_PENDING_COMACTION_BYTES_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_pending_compaction_bytes",
            "Pending compaction bytes",
            &["cf"]
        ).unwrap();

    pub static ref STORE_ENGINE_COMPACTION_FLOW_VEC: GaugeVec =
        register_gauge_vec!(
            "tikv_engine_compaction_flow_bytes",
            "Bytes of read/written during compaction",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_COMPACTION_DROP_VEC: CounterVec =
        register_counter_vec!(
            "tikv_engine_compaction_key_drop",
            "Count the reasons for key drop during compaction",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_COMPACTION_DURATIONS_VEC: HistogramVec =
        register_histogram_vec!(
            "tikv_engine_compaction_duration_seconds",
            "Histogram of compaction duration seconds",
            &["cf"],
            exponential_buckets(0.005, 2.0, 20).unwrap()
        ).unwrap();

    pub static ref STORE_ENGINE_COMPACTION_NUM_CORRUPT_KEYS_VEC: CounterVec =
        register_counter_vec!(
            "tikv_engine_compaction_num_corrupt_keys",
            "Number of corrupt keys during compaction",
            &["cf"]
        ).unwrap();

    pub static ref STORE_ENGINE_LOCATE_VEC: CounterVec =
        register_counter_vec!(
            "tikv_engine_locate",
            "Number of calls to seek/next/prev",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_FILE_STATUS_VEC: CounterVec =
        register_counter_vec!(
            "tikv_engine_file_status",
            "Number of different status of files",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_READ_AMP_FLOW_VEC: CounterVec =
        register_counter_vec!(
            "tikv_engine_read_amp_flow_bytes",
            "Bytes of read amplification",
            &["type"]
        ).unwrap();

    pub static ref STORE_ENGINE_NO_ITERATORS: Gauge =
        register_gauge!(
            "tikv_engine_no_iterator",
            "Number of iterators currently open"
        ).unwrap();

    pub static ref STORE_ENGINE_WAL_FILE_SYNCED: Counter =
        register_counter!(
            "tikv_engine_wal_file_synced",
            "Number of times WAL sync is done"
        ).unwrap();

    pub static ref STORE_ENGINE_EVENT_COUNTER_VEC: CounterVec =
        register_counter_vec!(
            "tikv_engine_event_total",
            "Number of engine events",
            &["cf", "type"]
        ).unwrap();
}
