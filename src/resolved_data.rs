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

    pub(crate) fn pretty_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dummy = Vec::new();
        let target_a = self.resp.get(&QueryType::A).unwrap_or(&dummy);
        let target_aaaa = self.resp.get(&QueryType::AAAA).unwrap_or(&dummy);
        write!(f, "<{}> {} =>", self.req_qtype, self.req_name)?;
        if !target_a.is_empty() {
            write!(f, " {}({})", QueryType::A, target_a.join(", "))?;
        }
        if !target_aaaa.is_empty() {
            write!(f, " {}({})", QueryType::AAAA, target_aaaa.join(", "))?;
        }
        for item in self
            .resp
            .iter()
            .filter(|x| *x.0 != QueryType::A && *x.0 != QueryType::AAAA)
        {
            write!(f, " {}({})", item.0, item.1.len())?;
        }
        Ok(())
    }
}
