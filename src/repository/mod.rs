use crate::model::domain::{CacheClient, DbClient};

pub mod cache;
pub mod db;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Repository {
    pub cache: cache::Cache,
    pub db: db::Database,
}

impl Repository {
    pub fn new(cache: CacheClient, db: DbClient) -> Self {
        Self { cache: cache::Cache::new(cache.clone()), db: db::Database::new(db.clone()) }
    }
}
