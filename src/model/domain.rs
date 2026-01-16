// src/model/domain.rs
use crate::{
    repository::Repository,
    utils::{fetch::Fetch, prometheus::PromOpts},
};
use deadpool_redis::Pool as RedisPool;
use sqlx::{MySql, Pool as MysqlPool};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub env: String,
    pub fetch: Fetch,
    pub prometheus: Arc<PromOpts>,
    pub repository: Repository,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DbClient {
    pub relation: DbManager,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct DbManager {
    pub master: MysqlPool<MySql>,
    pub slave: MysqlPool<MySql>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CacheClient {
    pub profile: RedisPool,
}
