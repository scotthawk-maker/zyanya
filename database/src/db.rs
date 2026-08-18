use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

pub use conn_builder::ConnBuilder;
use zyanya_utils::fd_budget::FDGuard;

mod conn_builder;

/// The DB type used for Zyanyad stores
pub struct DB {
    inner: DBWithThreadMode<MultiThreaded>,
    _fd_guard: FDGuard,
}

impl DB {
    pub fn new(inner: DBWithThreadMode<MultiThreaded>, fd_guard: FDGuard) -> Self {
        Self { inner, _fd_guard: fd_guard }
    }
}

impl DerefMut for DB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Deref for DB {
    type Target = DBWithThreadMode<MultiThreaded>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Deletes an existing DB if it exists.
///
/// F-L-36: returns a  instead of panicking on RocksDB destroy failure
/// or invalid path encoding.
pub fn delete_db(db_dir: PathBuf) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if !db_dir.exists() {
        return Ok(());
    }
    let options = rocksdb::Options::default();
    let path = db_dir.to_str().ok_or_else(|| format!("delete_db: path contains invalid UTF-8: {:?}", db_dir))?;
    <DBWithThreadMode<MultiThreaded>>::destroy(&options, path).map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    Ok(())
}
