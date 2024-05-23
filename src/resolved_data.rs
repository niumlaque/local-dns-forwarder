use crate::dns::QueryType;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug)]
pub struct ResolvedData {
    pub req_qtype: QueryType,
    pub req_name: String,
    pub resp: BTreeMap<QueryType, Vec<String>>,
}

impl ResolvedData {
    pub(crate) fn new(req_qtype: QueryType, req_name: impl Into<String>) -> Self {
        Self {
            req_qtype,
            req_name: req_name.into(),
            resp: Default::default(),
        }
    }

    pub(crate) fn insert(&mut self, resp_qtype: QueryType, resp_name: impl Into<String>) {
        self.resp
            .entry(resp_qtype)
            .or_default()
            .push(resp_name.into());
    }

    pub(crate) fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dummy = Vec::new();
        let mut set = HashSet::new();
        write!(f, "<{}> {} =>", self.req_qtype, self.req_name)?;

        match self.req_qtype {
            QueryType::A | QueryType::AAAA => {
                let target_a = self.resp.get(&QueryType::A).unwrap_or(&dummy);
                let target_aaaa = self.resp.get(&QueryType::AAAA).unwrap_or(&dummy);
                if !target_a.is_empty() {
                    write!(f, " {}({})", QueryType::A, target_a.join(", "))?;
                    set.insert(QueryType::A);
                }
                if !target_aaaa.is_empty() {
                    write!(f, " {}({})", QueryType::AAAA, target_aaaa.join(", "))?;
                    set.insert(QueryType::AAAA);
                }
            }
            QueryType::SRV => {
                let target_srv = self.resp.get(&QueryType::SRV).unwrap_or(&dummy);
                if !target_srv.is_empty() {
                    write!(f, " {}({})", QueryType::SRV, target_srv.join(", "))?;
                    set.insert(QueryType::SRV);
                }
            }
            _ => (),
        }
        for item in self.resp.iter().filter(|x| !set.contains(x.0)) {
            write!(f, " {}({})", item.0, item.1.len())?;
        }
        Ok(())
    }
}
