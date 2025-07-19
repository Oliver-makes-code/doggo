use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, LazyLock, Mutex},
};

use serde::{Deserialize, Serialize};

/// A pool for immutable interned strings
pub struct StringPool {
    value_map: Mutex<HashMap<Arc<str>, usize>>,
    values: Mutex<Vec<Option<StringPoolValue>>>,
    free_indices: Mutex<Vec<usize>>,
}

struct StringPoolValue {
    value: Arc<str>,
    ref_count: usize,
}

#[derive(PartialOrd)]
pub struct StrReference {
    index: usize,
}

static GLOBAL_POOL: LazyLock<StringPool> = LazyLock::new(StringPool::new);

impl StringPool {
    pub fn global() -> &'static Self {
        return &GLOBAL_POOL;
    }

    fn new() -> Self {
        return Self {
            value_map: Mutex::new(HashMap::new()),
            values: Mutex::new(vec![]),
            free_indices: Mutex::new(vec![]),
        };
    }

    pub fn acquire<'a>(&'a self, s: Arc<str>) -> Result<StrReference, Box<dyn Error + 'a>> {
        let mut value_map = self.value_map.lock()?;

        let mut values = self.values.lock()?;

        if let Some(index) = value_map.get(&s) {
            let Some(Some(value)) = values.get_mut(*index) else {
                return Err("Invalid index in values".into());
            };

            value.ref_count += 1;

            return Ok(StrReference { index: *index });
        }

        {
            let mut indices = self.free_indices.lock()?;

            if let Some(index) = indices.pop() {
                values[index] = Some(StringPoolValue {
                    value: s.clone(),
                    ref_count: 1,
                });

                value_map.insert(s, index);

                return Ok(StrReference { index });
            }
        }

        let index = values.len();

        values.push(Some(StringPoolValue {
            value: s.clone(),
            ref_count: 1,
        }));

        value_map.insert(s, index);

        return Ok(StrReference { index });
    }

    fn clone_reference<'a>(&'a self, index: usize) -> Result<StrReference, Box<dyn Error + 'a>> {
        let mut values = self.values.lock()?;

        let Some(Some(value)) = values.get_mut(index) else {
            return Err("Invalid index in values".into());
        };

        value.ref_count += 1;

        return Ok(StrReference { index });
    }

    fn get<'a>(&'a self, index: usize) -> Result<Option<Arc<str>>, Box<dyn Error + 'a>> {
        let values = self.values.lock()?;

        let Some(Some(value)) = values.get(index) else {
            return Ok(None);
        };

        return Ok(Some(value.value.clone()));
    }

    fn drop_reference<'a>(&'a self, index: usize) -> Result<(), Box<dyn Error + 'a>> {
        let mut values = self.values.lock()?;

        let Some(Some(value)) = values.get_mut(index) else {
            return Ok(());
        };

        value.ref_count -= 1;

        if value.ref_count != 0 {
            return Ok(());
        }

        let mut value_map = self.value_map.lock()?;
        let mut indices = self.free_indices.lock()?;

        value_map.remove(&value.value);

        indices.push(index);

        values[index] = None;

        return Ok(());
    }
}

impl StrReference {
    pub fn get(&self) -> Arc<str> {
        return StringPool::global().get(self.index).unwrap().unwrap();
    }
}

impl Drop for StrReference {
    fn drop(&mut self) {
        StringPool::global().drop_reference(self.index).unwrap();
    }
}

impl Clone for StrReference {
    fn clone(&self) -> Self {
        return StringPool::global().clone_reference(self.index).unwrap();
    }
}

impl Debug for StrReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("{:?}", self.get()));
    }
}

impl Display for StrReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("{}", self.get()));
    }
}

impl PartialEq for StrReference {
    fn eq(&self, other: &Self) -> bool {
        return self.index == other.index;
    }
}

impl Eq for StrReference {}

impl Hash for StrReference {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl Serialize for StrReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        return self.get().serialize(serializer);
    }
}

impl<'de> Deserialize<'de> for StrReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        return Ok(StringPool::global()
            .acquire(String::deserialize(deserializer)?.into())
            .unwrap());
    }
}
