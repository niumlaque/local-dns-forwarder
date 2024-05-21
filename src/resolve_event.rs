use crate::resolved_status::ResolvedStatus;

pub trait ResolveEvent {
    fn resolving(&self, name: &str);
    fn resolved(&self, status: ResolvedStatus);
    fn error(&self, _message: impl AsRef<str>) {}
}

pub struct DefaultResolveEvent;

impl ResolveEvent for DefaultResolveEvent {
    fn resolving(&self, name: &str) {
        println!("[Resolving] {name}");
    }

    fn resolved(&self, status: ResolvedStatus) {
        match status {
            ResolvedStatus::Allow(qtype, name, v) => println!(
                "[Allow] <{qtype}> {name}: {}",
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ResolvedStatus::AllowButError(qtype, name, code) => {
                println!("[Allow] <{qtype}> {name}: {code}")
            }
            ResolvedStatus::Deny(qtype, name, code) => println!("[Deny] <{qtype}> {name}: {code}"),
        }
    }

    fn error(&self, message: impl AsRef<str>) {
        println!("{}", message.as_ref());
    }
}

pub struct TracingResolveEvent;
impl ResolveEvent for TracingResolveEvent {
    fn resolving(&self, name: &str) {
        tracing::info!("[Resolving] {name}");
    }

    fn resolved(&self, status: ResolvedStatus) {
        match status {
            ResolvedStatus::Allow(qtype, name, v) => tracing::info!(
                "[Allow] <{qtype}> {name}: {}",
                v.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ResolvedStatus::AllowButError(qtype, name, code) => {
                tracing::info!("[Allow] <{qtype}> {name}: {code}")
            }
            ResolvedStatus::Deny(qtype, name, code) => {
                tracing::info!("[Deny] <{qtype}> {name}: {code}")
            }
        }
    }

    fn error(&self, message: impl AsRef<str>) {
        tracing::error!("{}", message.as_ref());
    }
}
