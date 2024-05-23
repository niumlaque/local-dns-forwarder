use crate::dns::QueryType;
use std::collections::BTreeMap;

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
}
