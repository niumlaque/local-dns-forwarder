use super::CheckList;

#[derive(Debug, PartialEq, Eq)]
pub enum CheckStatus {
    NotFound,
    Allow,
    Deny,
}

#[derive(Default)]
pub struct CompositeCheckList {
    pub allowlist: CheckList,
    pub denylist: CheckList,
}

impl CompositeCheckList {
    pub fn new(allowlist: CheckList, denylist: CheckList) -> Self {
        Self {
            allowlist,
            denylist,
        }
    }

    pub fn check(&self, name: &str) -> CheckStatus {
        if self.denylist.check(name) {
            // FQDN registered in the denylist is denied even if it's in the allowlist
            CheckStatus::Deny
        } else if self.allowlist.check(name) {
            // FQDN not in the denylist but registered in the allowlist is allowed
            CheckStatus::Allow
        } else {
            // FQDN not in either the denylist or the allowlist is denied
            CheckStatus::NotFound
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check() {
        let mut allowlist = CheckList::in_memory();
        allowlist.add("example.com");
        allowlist.add("example.org");

        let mut denylist = CheckList::in_memory();
        denylist.add("example.org");

        let list = CompositeCheckList::new(allowlist, denylist);
        assert_eq!(CheckStatus::Deny, list.check("example.org"));
        assert_eq!(CheckStatus::Allow, list.check("example.com"));
        assert_eq!(CheckStatus::NotFound, list.check("example.net"));
    }
}
