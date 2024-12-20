use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{api::WorkerBindingType, worker_binding::CompiledGolemWorkerBinding};
use golem_service_base::model::VersionedComponentId;
use rib::Expr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct GolemWorkerBinding {
    pub component_id: VersionedComponentId,
    pub worker_name: Expr,
    pub idempotency_key: Option<Expr>,
    pub response: ResponseMapping,
    #[serde(default)]
    pub binding_type: Option<WorkerBindingType>,
}

// ResponseMapping will consist of actual logic such as invoking worker functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct ResponseMapping(pub Expr);

impl From<CompiledGolemWorkerBinding> for GolemWorkerBinding {
    fn from(value: CompiledGolemWorkerBinding) -> Self {
        let worker_binding = value.clone();

        GolemWorkerBinding {
            component_id: worker_binding.component_id,
            worker_name: worker_binding.worker_name_compiled.worker_name,
            idempotency_key: worker_binding
                .idempotency_key_compiled
                .map(|idempotency_key_compiled| idempotency_key_compiled.idempotency_key),
            response: ResponseMapping(worker_binding.response_compiled.response_rib_expr),
            binding_type: Some(worker_binding.binding_type),
        }
    }
}
