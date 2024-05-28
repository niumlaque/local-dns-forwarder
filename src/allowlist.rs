use crate::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug)]
pub struct AllowList {
    inner: InMemoryAllowList,
}

impl AllowList {
    pub fn in_memory() -> Self {
        Self {
            inner: InMemoryAllowList::new(),
        }
    }

    pub fn text(path: PathBuf) -> Result<Self> {
        Ok(Self {
            inner: InMemoryAllowList::from_file(path)?,
        })
    }

    pub fn check(&self, name: &str) -> bool {
        self.inner.check(name)
    }

    pub fn add(&mut self, name: &str) -> usize {
        self.inner.add(name)
    }

    pub fn delete(&mut self, name: &str) -> usize {
        self.inner.delete(name)
    }

    pub fn count(&self) -> usize {
        self.inner.count()
    }
}

impl Default for AllowList {
    fn default() -> Self {
        Self::in_memory()
    }
}

#[derive(Debug, Default)]
pub struct InMemoryAllowList {
    #[allow(dead_code)]
    path: Option<PathBuf>,
    names: HashMap<String, ()>,
}

impl InMemoryAllowList {
    pub fn new() -> Self {
        Self {
            path: None,
            names: Default::default(),
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self> {
        let mut names = HashMap::new();
        for line in BufReader::new(File::open(&path)?).lines() {
            names.insert(line?, ());
        }

        Ok(Self {
            path: Some(path),
            names,
        })
    }

    pub fn check(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    pub fn add(&mut self, name: &str) -> usize {
        if self.names.contains_key(name) {
            0
        } else {
            self.names.insert(name.to_string(), ());
            1
        }
    }

    pub fn delete(&mut self, name: &str) -> usize {
        if self.names.remove(name).is_some() {
            1
        } else {
            0
        }
    }

    pub fn count(&self) -> usize {
        self.names.len()
    }
}
