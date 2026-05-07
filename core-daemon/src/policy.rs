use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum ApplyPolicy {
    AllDevices,
    SelectedDevices(HashSet<String>),
}

#[derive(Debug, Clone)]
pub struct PolicyEngine {
    apply_policy: ApplyPolicy,
    outbound_targets: HashSet<String>,
}

impl PolicyEngine {
    pub fn from_env() -> Self {
        let selected = parse_peer_list("CLIPPY_SELECTED_PEERS");
        let outbound = parse_peer_list("CLIPPY_TARGET_PEERS");

        let apply_policy = if selected.is_empty() {
            ApplyPolicy::AllDevices
        } else {
            ApplyPolicy::SelectedDevices(selected.clone())
        };

        Self {
            apply_policy,
            outbound_targets: outbound,
        }
    }

    pub fn outbound_targets(&self) -> Vec<String> {
        self.outbound_targets.iter().cloned().collect()
    }

    pub fn should_apply_message(&self, local_peer_id: &str, target_peer_ids: &[String]) -> bool {
        if !target_peer_ids.is_empty() {
            return target_peer_ids.iter().any(|peer| peer == local_peer_id);
        }

        match &self.apply_policy {
            ApplyPolicy::AllDevices => true,
            ApplyPolicy::SelectedDevices(selected) => selected.contains(local_peer_id),
        }
    }
}

fn parse_peer_list(var_name: &str) -> HashSet<String> {
    std::env::var(var_name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
