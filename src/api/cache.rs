use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs;
use std::path;
use std::time::{SystemTime, UNIX_EPOCH};

const CACHE_VERSION: &str = "v1";
const CACHE_EXPIRE_DAYS: u64 = 32;

#[derive(Serialize, Deserialize)]
pub struct CacheEntity {
    version: String,
    expire: u64,
    content: String,
}

pub struct Cache {
    file_path: path::PathBuf,
    use_cache: bool,
}

impl Cache {
    pub fn new(file_path: path::PathBuf, use_cache: bool) -> Self {
        if !use_cache {
            return Cache {
                file_path: path::PathBuf::new(),
                use_cache: false,
            };
        }
        let dir = file_path.parent().expect(&format!(
            "failed to get parent dir from '{}'",
            file_path.display()
        ));
        fs::create_dir_all(dir).expect(&format!("failed to create cache dir '{}'", dir.display()));
        if !file_path.exists() {
            fs::File::create(&file_path).expect(&format!(
                "failed to create cache file '{}'",
                file_path.display()
            ));
        }
        Cache {
            file_path,
            use_cache: true,
        }
    }

    fn read_cache_content(&self) -> Result<HashMap<String, CacheEntity>, anyhow::Error> {
        let content = fs::read_to_string(&self.file_path)?;
        if content.is_empty() {
            return Ok(HashMap::new());
        }
        let cache_entities = serde_json::from_str::<HashMap<String, CacheEntity>>(&content)?;
        Ok(cache_entities)
    }
}

pub trait Caching {
    fn write(&self, key: &str, content: String) -> anyhow::Result<()>;
    fn read(&self, key: &str) -> anyhow::Result<String>;
}

impl Caching for Cache {
    fn write(&self, key: &str, content: String) -> anyhow::Result<()> {
        if !self.use_cache {
            return Ok(());
        }
        let mut cache_entities = self.read_cache_content()?;
        let time_now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        cache_entities.insert(
            key.to_owned(),
            CacheEntity {
                version: CACHE_VERSION.to_owned(),
                expire: time_now.as_secs() + CACHE_EXPIRE_DAYS * 24 * 60 * 60,
                content,
            },
        );
        let new_file_content = serde_json::to_string(&cache_entities)?;
        fs::write(&self.file_path, new_file_content)?;
        Ok(())
    }

    fn read(&self, key: &str) -> anyhow::Result<String> {
        if !self.use_cache {
            return Err(anyhow!("cache is disabled on purpose"));
        }
        let mut cache_entities = match self.read_cache_content() {
            Ok(val) => val,
            Err(e) => {
                return Err(e);
            }
        };
        let cache_entity = match cache_entities.remove(key) {
            Some(val) => val,
            None => {
                return Err(anyhow!("'{}' not found in the cache", key));
            }
        };
        if cache_entity.version != CACHE_VERSION {
            return Err(anyhow!(
                "'{}' is not the right cache version now",
                cache_entity.version
            ));
        }
        let time_now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(val) => val,
            Err(e) => {
                return Err(e.into());
            }
        };
        if time_now.as_secs() > cache_entity.expire {
            return Err(anyhow!("cache expired"));
        }
        Ok(cache_entity.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    // Since it's a bit tricky to mock time in rust, so we will just set a time range for during the test.
    // there is no chance a test is taking longer than 10 minutes.
    pub const TIME_EPS_SECONDS: u64 = 10 * 60;

    lazy_static! {
        static ref CACHE_DIR: std::path::PathBuf = std::env::temp_dir().join("test_write");
    }

    pub fn prepare_cache(sub_dir: &str, use_cache: bool) -> Cache {
        let dir = CACHE_DIR.clone().join(sub_dir);
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        let file_path = dir.join("cache.json");
        Cache::new(file_path.clone(), use_cache)
    }

    pub fn get_expected_expire() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 24 * 60 * 60 * CACHE_EXPIRE_DAYS
    }

    mod write {
        use super::*;

        #[test]
        fn write_to_empty_file() {
            let expire = get_expected_expire();
            let cache = prepare_cache("write/write_to_empty_file", true);

            cache.write("foo", "bar".to_owned()).unwrap();
            let cache_entities = cache.read_cache_content().unwrap();
            assert_eq!(cache_entities.len(), 1);
            let foo = cache_entities
                .get("foo")
                .expect("foo should exist in the cache");
            assert_eq!(foo.content, "bar");
            assert_eq!(foo.version, CACHE_VERSION);
            assert!(foo.expire >= expire && foo.expire < expire + TIME_EPS_SECONDS);
        }

        #[test]
        fn write_to_file_with_other_cache() {
            let expire = get_expected_expire();
            let cache = prepare_cache("write/write_to_file_with_other_cache", true);

            fs::write(
                &cache.file_path,
                r#"{ "baz": { "version": "v123456", "expire": 0, "content": "foobar" } }"#,
            )
            .unwrap();

            cache.write("foo", "bar".to_owned()).unwrap();
            let cache_entities = cache.read_cache_content().unwrap();
            assert_eq!(cache_entities.len(), 2);
            let foo = cache_entities
                .get("foo")
                .expect("foo should exist in the cache");
            assert_eq!(foo.content, "bar");
            assert_eq!(foo.version, CACHE_VERSION);
            assert!(foo.expire >= expire && foo.expire < expire + TIME_EPS_SECONDS);
            let baz = cache_entities
                .get("baz")
                .expect("bar should exist in the cache");
            assert_eq!(baz.content, "foobar");
            assert_eq!(baz.version, "v123456");
            assert_eq!(baz.expire, 0);
        }

        #[test]
        fn write_to_file_overwrite() {
            let expire = get_expected_expire();
            let cache = prepare_cache("write/write_to_file_overwrite", true);

            fs::write(
                &cache.file_path,
                r#"{
                "foo": { "version": "v1234", "expire": 1, "content": "foo" },
                "baz": { "version": "v123456", "expire": 0, "content": "foobar" }
              }"#,
            )
            .unwrap();

            cache.write("foo", "bar".to_owned()).unwrap();
            let cache_entities = cache.read_cache_content().unwrap();
            assert_eq!(cache_entities.len(), 2);
            let foo = cache_entities
                .get("foo")
                .expect("foo should exist in the cache");
            assert_eq!(foo.content, "bar");
            assert_eq!(foo.version, CACHE_VERSION);
            assert!(foo.expire >= expire && foo.expire < expire + TIME_EPS_SECONDS);
            let baz = cache_entities
                .get("baz")
                .expect("bar should exist in the cache");
            assert_eq!(baz.content, "foobar");
            assert_eq!(baz.version, "v123456");
            assert_eq!(baz.expire, 0);
        }

        #[test]
        fn no_cache_not_write_anything() {
            let cache = prepare_cache("write/no_cache_not_write_anything", false);
            let path = CACHE_DIR
                .clone()
                .join("write/no_cache_not_write_anything")
                .join("cache.json");
            assert!(!path.exists());
            cache.write("foo", "bar".to_owned()).unwrap();
            assert!(!path.exists());
        }
    }

    mod read {
        use super::*;

        #[test]
        fn read_json() {
            let expire = get_expected_expire();
            let cache = prepare_cache("read/read_json", true);

            fs::write(
                &cache.file_path,
                serde_json::to_string(&HashMap::from([(
                    "foo",
                    CacheEntity {
                        version: CACHE_VERSION.to_owned(),
                        expire,
                        content: "bar".to_owned(),
                    },
                )]))
                .unwrap(),
            )
            .unwrap();
            cache.read("foo").expect("failed to read cache value");
        }

        #[test]
        #[should_panic(expected = "'v_invalid_version' is not the right cache version now")]
        fn wrong_version() {
            let expire = get_expected_expire();
            let cache = prepare_cache("read/wrong_version", true);

            fs::write(
                &cache.file_path,
                serde_json::to_string(&HashMap::from([(
                    "foo",
                    CacheEntity {
                        version: "v_invalid_version".to_owned(),
                        expire,
                        content: "bar".to_owned(),
                    },
                )]))
                .unwrap(),
            )
            .unwrap();
            cache.read("foo").unwrap();
        }

        #[test]
        #[should_panic(expected = "cache is disabled on purpose")]
        fn not_use_cache() {
            let cache = prepare_cache("read/not_use_cache", false);
            cache.read("foo").unwrap();
        }

        #[test]
        #[should_panic(expected = "'bar' not found in the cache")]
        fn missing() {
            let expire = get_expected_expire();
            let cache = prepare_cache("read/missing", true);

            fs::write(
                &cache.file_path,
                serde_json::to_string(&HashMap::from([(
                    "foo",
                    CacheEntity {
                        version: "v_invalid_version".to_owned(),
                        expire,
                        content: "bar".to_owned(),
                    },
                )]))
                .unwrap(),
            )
            .unwrap();
            cache.read("bar").unwrap();
        }

        #[test]
        #[should_panic(expected = "cache expired")]
        fn expire() {
            let expire = get_expected_expire();
            let cache = prepare_cache("read/expire", true);

            fs::write(
                &cache.file_path,
                serde_json::to_string(&HashMap::from([(
                    "foo",
                    CacheEntity {
                        version: CACHE_VERSION.to_owned(),
                        expire: expire - (24 + 1) * 60 * 60 * CACHE_EXPIRE_DAYS,
                        content: "bar".to_owned(),
                    },
                )]))
                .unwrap(),
            )
            .unwrap();
            cache.read("foo").unwrap();
        }
    }
}
