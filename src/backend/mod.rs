use std::sync::Arc;

use dashmap::DashMap;
use derive_more::derive::Deref;

use crate::RespFrame;

#[derive(Debug, Clone, Deref, Default)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug, Default)]
pub struct BackendInner {
    pub map: DashMap<String, RespFrame>,
    pub hmap: DashMap<String, DashMap<String, RespFrame>>,
}

impl Backend {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn hset(&self, table_name: String, key: String, value: RespFrame) {
        let target_table = self.hmap.entry(table_name).or_default();
        target_table.insert(key, value);
    }

    pub fn hget(&self, table_name: &str, key: &str) -> Option<RespFrame> {
        self.hmap
            .get(table_name)
            .and_then(|v| v.get(key).map(|v| v.value().clone()))
    }

    pub fn hgetall(&self, table_name: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap
            .get(table_name)
            // .and_then(|target_table| Some(target_table.clone())) //and_then方法也可行，但是需要手动用Some包装起来成为Option类型
            .map(|target_table| target_table.clone()) //map方法更简洁，会自动包装为Option类型
    }
}
