use crate::{Error, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use wildmatch::WildMatch;

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

    pub fn save(&self) -> Result<()> {
        self.inner.save()
    }

    pub fn iter(&self) -> AllowListIterator<impl std::iter::Iterator<Item = &str>> {
        AllowListIterator {
            source: self.inner.iter(),
        }
    }
}

impl Default for AllowList {
    fn default() -> Self {
        Self::in_memory()
    }
}

pub struct AllowListIterator<I: std::iter::Iterator> {
    source: I,
}

impl<'a, I: std::iter::Iterator<Item = &'a str>> Iterator for AllowListIterator<I> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }
}

#[derive(Debug, Default)]
pub struct InMemoryAllowList {
    #[allow(dead_code)]
    path: Option<PathBuf>,
    names: HashMap<String, ()>,
    wnames: HashMap<String, WildMatch>,
}

impl InMemoryAllowList {
    pub fn new() -> Self {
        Self {
            path: None,
            names: Default::default(),
            wnames: Default::default(),
        }
    }

    pub fn from_file(path: PathBuf) -> Result<Self> {
        let mut names = HashMap::new();
        let mut wnames = HashMap::new();
        for line in BufReader::new(File::open(&path)?).lines() {
            let line = line?;
            if line.contains('*') {
                let w = WildMatch::new(&line);
                wnames.insert(line, w);
            } else {
                names.insert(line, ());
            }
        }

        Ok(Self {
            path: Some(path),
            names,
            wnames,
        })
    }

    pub fn check(&self, name: &str) -> bool {
        if self.names.contains_key(name) {
            true
        } else {
            self.wnames.values().any(|x| x.matches(name))
        }
    }

    pub fn add(&mut self, name: &str) -> usize {
        use std::collections::hash_map::Entry::Vacant;
        if name.contains('*') {
            if let Vacant(e) = self.wnames.entry(name.to_string()) {
                e.insert(WildMatch::new(name));
                1
            } else {
                0
            }
        } else if !self.names.contains_key(name) {
            self.names.insert(name.to_string(), ());
            1
        } else {
            0
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
        self.names.len() + self.wnames.len()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = self.path.as_ref() {
            let mut names = self.names.keys().collect::<Vec<_>>();
            names.sort();
            let mut wnames = self.wnames.keys().collect::<Vec<_>>();
            wnames.sort();
            let f = File::create(path)?;
            let mut w = BufWriter::new(f);
            for name in names {
                writeln!(w, "{}", name)?;
            }
            for wname in wnames {
                writeln!(w, "{}", wname)?;
            }

            w.flush()?;
            Ok(())
        } else {
            Err(Error::SaveButInMemory)
        }
    }

    pub fn iter(&self) -> InMemoryAllowListIterator<'_> {
        InMemoryAllowListIterator {
            names_keys: self.names.keys(),
            wnames_keys: self.wnames.keys(),
        }
    }
}

pub struct InMemoryAllowListIterator<'a> {
    names_keys: std::collections::hash_map::Keys<'a, String, ()>,
    wnames_keys: std::collections::hash_map::Keys<'a, String, WildMatch>,
}

impl<'a> Iterator for InMemoryAllowListIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(key) = self.names_keys.next() {
            return Some(key.as_str());
        } else if let Some(key) = self.wnames_keys.next() {
            return Some(key.as_str());
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inmemory_al_data() {
        let mut m = InMemoryAllowList::new();
        assert_eq!(None, m.path);
        assert_eq!(0, m.names.len());
        assert_eq!(0, m.wnames.len());
        assert_eq!(0, m.count());

        let name1 = "www.example.com";
        assert_eq!(1, m.add(name1));
        assert_eq!(1, m.names.len());
        assert_eq!(0, m.wnames.len());
        assert_eq!(1, m.count());
        assert!(m.names.contains_key(name1));
        assert_eq!(0, m.add(name1));
        assert_eq!(1, m.names.len());
        assert_eq!(0, m.wnames.len());
        assert_eq!(1, m.count());

        let name2 = "www.gnu.org";
        assert_eq!(1, m.add(name2));
        assert_eq!(2, m.names.len());
        assert_eq!(0, m.wnames.len());
        assert_eq!(2, m.count());
        assert!(m.names.contains_key(name1));
        assert!(m.names.contains_key(name2));
        assert_eq!(0, m.add(name2));
        assert_eq!(2, m.names.len());
        assert_eq!(0, m.wnames.len());
        assert_eq!(2, m.count());

        let name3 = "example.*";
        assert_eq!(1, m.add(name3));
        assert_eq!(2, m.names.len());
        assert_eq!(1, m.wnames.len());
        assert_eq!(3, m.count());
        assert!(m.names.contains_key(name1));
        assert!(m.names.contains_key(name2));
        assert!(m.wnames.contains_key(name3));
        assert_eq!(0, m.add(name3));
        assert_eq!(2, m.names.len());
        assert_eq!(1, m.wnames.len());
        assert_eq!(3, m.count());

        let name4 = "*.debian.org";
        assert_eq!(1, m.add(name4));
        assert_eq!(2, m.names.len());
        assert_eq!(2, m.wnames.len());
        assert_eq!(4, m.count());
        assert!(m.names.contains_key(name1));
        assert!(m.names.contains_key(name2));
        assert!(m.wnames.contains_key(name3));
        assert!(m.wnames.contains_key(name4));
        assert_eq!(0, m.add(name4));
        assert_eq!(2, m.names.len());
        assert_eq!(2, m.wnames.len());
        assert_eq!(4, m.count());
    }

    #[test]
    fn test_inmemory_al_check() {
        let mut m = InMemoryAllowList::new();
        m.add("www.example.com");
        m.add("www.gnu.org");
        m.add("example.*");
        m.add("*.debian.org");

        assert!(m.check("www.example.com"));
        assert!(m.check("www.gnu.org"));
        assert!(m.check("example.org"));
        assert!(m.check("example.co.jp"));
        assert!(m.check("deb.debian.org"));
        assert!(m.check("ftp.jp.debian.org"));

        assert!(!m.check("example"));
        assert!(!m.check("www.example"));
        assert!(!m.check("debian.org"));
        assert!(!m.check("www.google.co.jp"));
    }
}
