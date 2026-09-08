use crate::db::DB;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::{path::PathBuf, sync::Arc};

/// F-M-38: error type returned by [`ConnBuilder::build`] instead of panicking
/// on a RocksDB open failure or a non-UTF-8 database path.
#[derive(Debug, thiserror::Error)]
pub enum ConnBuilderError {
    #[error("fd budget error: {0}")]
    FdBudget(#[from] zyanya_utils::fd_budget::Error),
    #[error("rocksdb open error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    #[error("database path is not valid UTF-8")]
    NonUtf8Path,
}

#[derive(Debug)]
pub struct Unspecified;

#[derive(Debug)]
pub struct ConnBuilder<Path, const STATS_ENABLED: bool, StatsPeriod, FDLimit> {
    db_path: Path,
    create_if_missing: bool,
    parallelism: usize,
    files_limit: FDLimit,
    mem_budget: usize,
    stats_period: StatsPeriod,
}

impl Default for ConnBuilder<Unspecified, false, Unspecified, Unspecified> {
    fn default() -> Self {
        ConnBuilder {
            db_path: Unspecified,
            create_if_missing: true,
            parallelism: 1,
            mem_budget: 64 * 1024 * 1024,
            stats_period: Unspecified,
            files_limit: Unspecified,
        }
    }
}

impl<Path, const STATS_ENABLED: bool, StatsPeriod, FDLimit> ConnBuilder<Path, STATS_ENABLED, StatsPeriod, FDLimit> {
    pub fn with_db_path(self, db_path: PathBuf) -> ConnBuilder<PathBuf, STATS_ENABLED, StatsPeriod, FDLimit> {
        ConnBuilder {
            db_path,
            files_limit: self.files_limit,
            create_if_missing: self.create_if_missing,
            parallelism: self.parallelism,
            mem_budget: self.mem_budget,
            stats_period: self.stats_period,
        }
    }
    pub fn with_create_if_missing(self, create_if_missing: bool) -> ConnBuilder<Path, STATS_ENABLED, StatsPeriod, FDLimit> {
        ConnBuilder { create_if_missing, ..self }
    }
    pub fn with_parallelism(self, parallelism: impl Into<usize>) -> ConnBuilder<Path, STATS_ENABLED, StatsPeriod, FDLimit> {
        ConnBuilder { parallelism: parallelism.into(), ..self }
    }
    pub fn with_mem_budget(self, mem_budget: impl Into<usize>) -> ConnBuilder<Path, STATS_ENABLED, StatsPeriod, FDLimit> {
        ConnBuilder { mem_budget: mem_budget.into(), ..self }
    }
    pub fn with_files_limit(self, files_limit: impl Into<i32>) -> ConnBuilder<Path, STATS_ENABLED, StatsPeriod, i32> {
        ConnBuilder {
            db_path: self.db_path,
            files_limit: files_limit.into(),
            create_if_missing: self.create_if_missing,
            parallelism: self.parallelism,
            mem_budget: self.mem_budget,
            stats_period: self.stats_period,
        }
    }
}

impl<Path, FDLimit> ConnBuilder<Path, false, Unspecified, FDLimit> {
    pub fn enable_stats(self) -> ConnBuilder<Path, true, Unspecified, FDLimit> {
        ConnBuilder {
            db_path: self.db_path,
            create_if_missing: self.create_if_missing,
            parallelism: self.parallelism,
            files_limit: self.files_limit,
            mem_budget: self.mem_budget,
            stats_period: self.stats_period,
        }
    }
}

impl<Path, StatsPeriod, FDLimit> ConnBuilder<Path, true, StatsPeriod, FDLimit> {
    pub fn disable_stats(self) -> ConnBuilder<Path, false, Unspecified, FDLimit> {
        ConnBuilder {
            db_path: self.db_path,
            create_if_missing: self.create_if_missing,
            parallelism: self.parallelism,
            files_limit: self.files_limit,
            mem_budget: self.mem_budget,
            stats_period: Unspecified,
        }
    }
    pub fn with_stats_period(self, stats_period: impl Into<u32>) -> ConnBuilder<Path, true, u32, FDLimit> {
        ConnBuilder {
            db_path: self.db_path,
            create_if_missing: self.create_if_missing,
            parallelism: self.parallelism,
            files_limit: self.files_limit,
            mem_budget: self.mem_budget,
            stats_period: stats_period.into(),
        }
    }
}

macro_rules! default_opts {
    ($self: expr) => {{
        let mut opts = rocksdb::Options::default();
        if $self.parallelism > 1 {
            opts.increase_parallelism($self.parallelism as i32);
        }

        opts.optimize_level_style_compaction($self.mem_budget);
        // F-M-38: explicitly map the fd-budget error to ConnBuilderError so the
        // `?` operator does not need to infer between multiple From impls.
        let guard = zyanya_utils::fd_budget::acquire_guard($self.files_limit).map_err(ConnBuilderError::FdBudget)?;
        opts.set_max_open_files($self.files_limit);
        opts.create_if_missing($self.create_if_missing);
        Ok::<_, ConnBuilderError>((opts, guard))
    }};
}

impl ConnBuilder<PathBuf, false, Unspecified, i32> {
    pub fn build(self) -> Result<Arc<DB>, ConnBuilderError> {
        let (opts, guard) = default_opts!(self)?;
        let path = self.db_path.to_str().ok_or(ConnBuilderError::NonUtf8Path)?;
        let db = Arc::new(DB::new(<DBWithThreadMode<MultiThreaded>>::open(&opts, path).map_err(ConnBuilderError::RocksDb)?, guard));
        Ok(db)
    }
}

impl ConnBuilder<PathBuf, true, Unspecified, i32> {
    pub fn build(self) -> Result<Arc<DB>, ConnBuilderError> {
        let (mut opts, guard) = default_opts!(self)?;
        opts.enable_statistics();
        let path = self.db_path.to_str().ok_or(ConnBuilderError::NonUtf8Path)?;
        let db = Arc::new(DB::new(<DBWithThreadMode<MultiThreaded>>::open(&opts, path).map_err(ConnBuilderError::RocksDb)?, guard));
        Ok(db)
    }
}

impl ConnBuilder<PathBuf, true, u32, i32> {
    pub fn build(self) -> Result<Arc<DB>, ConnBuilderError> {
        let (mut opts, guard) = default_opts!(self)?;
        opts.enable_statistics();
        opts.set_report_bg_io_stats(true);
        opts.set_stats_dump_period_sec(self.stats_period);
        let path = self.db_path.to_str().ok_or(ConnBuilderError::NonUtf8Path)?;
        let db = Arc::new(DB::new(<DBWithThreadMode<MultiThreaded>>::open(&opts, path).map_err(ConnBuilderError::RocksDb)?, guard));
        Ok(db)
    }
}
