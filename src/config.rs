// src/config.rs
#[derive(Clone, Debug)]
pub struct Config {
    pub cache: RedisConfig,
    pub db: MysqlConfig,
    pub env: String,
    pub log: String,
    pub port: String,
}

#[derive(Clone, Debug)]
pub struct RedisConfig {
    pub profile: String,
}

#[derive(Clone, Debug)]
pub struct MysqlConfig {
    pub relation: DbConfig,
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub master: String,
    pub slave: String,
}

#[derive(Clone, Debug)]
pub struct ConfMySQL {
    pub master: &'static str,
    pub slave: &'static str,
    pub username: &'static str,
    pub password: &'static str,
    pub database: &'static str,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ConfRedis {
    pub host: &'static str,
    pub password: &'static str,
}

impl Config {
    pub fn init() -> Self {
        let cache = Self::get_redis_config("default");
        let db = Self::get_mysql_config("default");

        let profile = Self::create_redis_uri(cache);
        let relation = Self::create_mysql_uri(db);

        Self {
            cache: RedisConfig { profile },
            db: MysqlConfig { relation },
            env: Self::get_mode(),
            log: std::env::var("LOG_DIR").unwrap_or_else(|_| "/data/logs/rust-practice".into()),
            port: std::env::var("KS_PORT").unwrap_or_else(|_| "8880".into()),
        }
    }

    fn get_mode() -> String {
        match std::env::var("RP_MODE") {
            Ok(v) if v == "release" || v == "test" => v,
            _ => "test".to_string(),
        }
    }

    fn create_mysql_uri(data: ConfMySQL) -> DbConfig {
        if data.username.is_empty()
            || data.password.is_empty()
            || data.master.is_empty()
            || data.slave.is_empty()
            || data.database.is_empty()
        {
            return DbConfig { master: String::new(), slave: String::new() };
        }

        return DbConfig {
            master: format!("mysql://{}:{}@{}/{}", data.username, data.password, data.master, data.database),
            slave: format!("mysql://{}:{}@{}/{}", data.username, data.password, data.slave, data.database),
        };
    }

    fn create_redis_uri(data: ConfRedis) -> String {
        if data.host.is_empty() {
            return String::new();
        }

        format!("redis://{}", data.host)
    }

    #[allow(dead_code)]
    fn create_redis_uri_with_password(data: ConfRedis) -> String {
        if data.host.is_empty() {
            return String::new();
        }

        match data.password {
            "" => format!("redis://{}", data.host),
            _ => format!("redis://:{}@{}", data.password, data.host),
        }
    }

    fn get_mysql_config(key: &str) -> ConfMySQL {
        let mode = Self::get_mode();

        match (key, mode.as_str()) {
            ("default", "release") => ConfMySQL {
                master: "127.0.0.1:3306",
                slave: "127.0.0.1:3306",
                username: "test",
                password: "test@123",
                database: "test",
            },
            ("default", "test") => ConfMySQL {
                master: "127.0.0.1:3306",
                slave: "127.0.0.1:3306",
                username: "test",
                password: "test@123",
                database: "test",
            },

            _ => panic!("Unknown database key: {}", key),
        }
    }

    fn get_redis_config(key: &str) -> ConfRedis {
        let mode = Self::get_mode();

        match (key, mode.as_str()) {
            ("default", "release") => ConfRedis { host: "127.0.0.1:6379", password: "" },
            ("default", "test") => ConfRedis { host: "127.0.0.1:6379", password: "" },

            _ => panic!("Unknown redis key: {}", key),
        }
    }
}
