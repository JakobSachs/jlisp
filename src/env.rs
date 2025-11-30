use slotmap::SlotMap;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::ast::Expr;

// i got this to work, but honestly dont 100% understand how this slotmap works
pub type EnvId = slotmap::DefaultKey;

#[derive(Debug, Clone)]
struct EnvData {
    pub map: HashMap<String, Expr>,
    pub parent: Option<EnvId>,
}

thread_local! {
    static ENV_STORAGE: RefCell<SlotMap<EnvId, EnvData>> = RefCell::new(SlotMap::with_key());
}

#[derive(Debug, Clone, Copy)]
pub struct Env(EnvId);

impl Env {
    pub fn new() -> Self {
        ENV_STORAGE.with(|storage| {
            let id = storage.borrow_mut().insert(EnvData {
                map: HashMap::new(),
                parent: None,
            });
            Env(id)
        })
    }

    pub fn child(parent: Env) -> Self {
        ENV_STORAGE.with(|storage| {
            let id = storage.borrow_mut().insert(EnvData {
                map: HashMap::new(),
                parent: Some(parent.0),
            });
            Env(id)
        })
    }

    pub fn get(&self, key: &str) -> Option<Expr> {
        ENV_STORAGE.with(|storage| {
            let storage = storage.borrow();
            let mut current = Some(self.0);
            while let Some(id) = current {
                let data = &storage[id];
                if let Some(v) = data.map.get(key) {
                    return Some(v.clone());
                }
                current = data.parent;
            }
            None
        })
    }

    pub fn insert(&self, key: String, val: Expr) {
        ENV_STORAGE.with(|storage| {
            storage.borrow_mut()[self.0].map.insert(key, val);
        })
    }

    pub fn root(&self) -> Env {
        ENV_STORAGE.with(|storage| {
            let storage = storage.borrow();
            let mut current = self.0;
            loop {
                let data = &storage[current];
                match data.parent {
                    Some(p) => current = p,
                    None => return Env(current),
                }
            }
        })
    }

    pub fn remove(&self, key: &str) {
        ENV_STORAGE.with(|storage| {
            storage.borrow_mut()[self.0].map.remove(key);
        })
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
