use bincode::{Decode, Encode};
use golem_api_grpc::proto::golem::apidefinition as grpc_apidefinition;
use golem_service_base::model::VersionedComponentId;
use poem_openapi::*;
use serde::{Deserialize, Serialize};
use std::result::Result;
use std::time::SystemTime;

use crate::api_definition::http::{
    AllPathPatterns, CompiledHttpApiDefinition, CompiledRoute, MethodPattern,
};
use crate::api_definition::{ApiDefinitionId, ApiSite, ApiVersion};
use crate::worker_binding::CompiledGolemWorkerBinding;
use rib::{Expr, RibInputTypeInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDeploymentRequest {
    pub api_definitions: Vec<ApiDefinitionInfo>,
    pub site: ApiSite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDeployment {
    pub api_definitions: Vec<ApiDefinitionInfo>,
    pub site: ApiSite,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct ApiDefinitionInfo {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
}

// Mostly this data structures that represents the actual incoming request
// exist due to the presence of complicated Expr data type in api_definition::ApiDefinition.
// Consider them to be otherwise same
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct HttpApiDefinitionRequest {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<Route>,
    #[serde(default)]
    pub draft: bool,
}

// Mostly this data structures that represents the actual incoming request
// exist due to the presence of complicated Expr data type in api_definition::ApiDefinition.
// Consider them to be otherwise same
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct HttpApiDefinition {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<Route>,
    #[serde(default)]
    pub draft: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// HttpApiDefinitionWithTypeInfo is CompiledHttpApiDefinition minus rib-byte-code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct HttpApiDefinitionWithTypeInfo {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<RouteWithTypeInfo>,
    #[serde(default)]
    pub draft: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<CompiledHttpApiDefinition> for HttpApiDefinitionWithTypeInfo {
    fn from(value: CompiledHttpApiDefinition) -> Self {
        let routes = value.routes.into_iter().map(|route| route.into()).collect();

        Self {
            id: value.id,
            version: value.version,
            routes,
            draft: value.draft,
            created_at: Some(value.created_at),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
pub struct Route {
    pub method: MethodPattern,
    pub path: String,
    pub binding: GolemWorkerBinding,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
pub struct RouteWithTypeInfo {
    pub method: MethodPattern,
    pub path: String,
    pub binding: GolemWorkerBindingWithTypeInfo,
}

impl From<CompiledRoute> for RouteWithTypeInfo {
    fn from(value: CompiledRoute) -> Self {
        let method = value.method;
        let path = value.path.to_string();
        let binding = value.binding.into();
        Self {
            method,
            path,
            binding,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct GolemWorkerBinding {
    pub component_id: VersionedComponentId,
    pub worker_name: String,
    pub idempotency_key: Option<String>,
    pub response: String,
    #[serde(default)]
    pub binding_type: Option<WorkerBindingType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
#[oai(rename_all = "camelCase")]
pub struct GolemWorkerBindingWithTypeInfo {
    pub component_id: VersionedComponentId,
    pub worker_name: String,
    pub idempotency_key: Option<String>,
    pub response: String,
    #[serde(default)]
    pub binding_type: Option<WorkerBindingType>,
    pub response_mapping_input: Option<RibInputTypeInfo>,
    pub worker_name_input: Option<RibInputTypeInfo>,
    pub idempotency_key_input: Option<RibInputTypeInfo>,
}

impl From<CompiledGolemWorkerBinding> for GolemWorkerBindingWithTypeInfo {
    fn from(value: CompiledGolemWorkerBinding) -> Self {
        let worker_binding = value.clone();

        GolemWorkerBindingWithTypeInfo {
            component_id: worker_binding.component_id,
            worker_name: worker_binding.worker_name_compiled.worker_name.to_string(),
            idempotency_key: worker_binding.idempotency_key_compiled.map(
                |idempotency_key_compiled| idempotency_key_compiled.idempotency_key.to_string(),
            ),
            response: worker_binding
                .response_compiled
                .response_rib_expr
                .to_string(),
            binding_type: Some(value.binding_type),
            response_mapping_input: Some(worker_binding.response_compiled.rib_input),
            worker_name_input: Some(worker_binding.worker_name_compiled.rib_input_type_info),
            idempotency_key_input: value
                .idempotency_key_compiled
                .map(|idempotency_key_compiled| idempotency_key_compiled.rib_input),
        }
    }
}

impl<N> From<crate::api_definition::ApiDeployment<N>> for ApiDeployment {
    fn from(value: crate::api_definition::ApiDeployment<N>) -> Self {
        let api_definitions = value
            .api_definition_keys
            .into_iter()
            .map(|key| ApiDefinitionInfo {
                id: key.id,
                version: key.version,
            })
            .collect();

        Self {
            api_definitions,
            site: value.site,
            created_at: Some(value.created_at),
        }
    }
}

impl TryFrom<crate::api_definition::http::HttpApiDefinition> for HttpApiDefinition {
    type Error = String;

    fn try_from(
        value: crate::api_definition::http::HttpApiDefinition,
    ) -> Result<Self, Self::Error> {
        let mut routes = Vec::new();
        for route in value.routes {
            let v = Route::try_from(route)?;
            routes.push(v);
        }

        Ok(Self {
            id: value.id,
            version: value.version,
            routes,
            draft: value.draft,
            created_at: Some(value.created_at),
        })
    }
}

impl TryInto<crate::api_definition::http::HttpApiDefinitionRequest> for HttpApiDefinitionRequest {
    type Error = String;

    fn try_into(
        self,
    ) -> Result<crate::api_definition::http::HttpApiDefinitionRequest, Self::Error> {
        let mut routes = Vec::new();

        for route in self.routes {
            let v = route.try_into()?;
            routes.push(v);
        }

        Ok(crate::api_definition::http::HttpApiDefinitionRequest {
            id: self.id,
            version: self.version,
            routes,
            draft: self.draft,
        })
    }
}

impl TryFrom<crate::api_definition::http::Route> for Route {
    type Error = String;

    fn try_from(value: crate::api_definition::http::Route) -> Result<Self, Self::Error> {
        let path = value.path.to_string();
        let binding = GolemWorkerBinding::try_from(value.binding)?;

        Ok(Self {
            method: value.method,
            path,
            binding,
        })
    }
}

impl TryInto<crate::api_definition::http::Route> for Route {
    type Error = String;

    fn try_into(self) -> Result<crate::api_definition::http::Route, Self::Error> {
        let path = AllPathPatterns::parse(self.path.as_str()).map_err(|e| e.to_string())?;
        let binding = self.binding.try_into()?;

        Ok(crate::api_definition::http::Route {
            method: self.method,
            path,
            binding,
        })
    }
}

impl TryFrom<crate::worker_binding::GolemWorkerBinding> for GolemWorkerBinding {
    type Error = String;

    fn try_from(value: crate::worker_binding::GolemWorkerBinding) -> Result<Self, Self::Error> {
        let response: String = rib::to_string(&value.response.0).map_err(|e| e.to_string())?;

        let worker_id = rib::to_string(&value.worker_name).map_err(|e| e.to_string())?;

        let idempotency_key = if let Some(key) = &value.idempotency_key {
            Some(rib::to_string(key).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let binding_type = value.binding_type.into();

        Ok(Self {
            component_id: value.component_id,
            worker_name: worker_id,
            idempotency_key,
            response,
            binding_type,
        })
    }
}

impl TryInto<crate::worker_binding::GolemWorkerBinding> for GolemWorkerBinding {
    type Error = String;

    fn try_into(self) -> Result<crate::worker_binding::GolemWorkerBinding, Self::Error> {
        let response: crate::worker_binding::ResponseMapping = {
            let r = rib::from_string(self.response.as_str()).map_err(|e| e.to_string())?;
            crate::worker_binding::ResponseMapping(r)
        };

        let worker_name: Expr =
            rib::from_string(self.worker_name.as_str()).map_err(|e| e.to_string())?;

        let idempotency_key = if let Some(key) = &self.idempotency_key {
            Some(rib::from_string(key).map_err(|e| e.to_string())?)
        } else {
            None
        };

        let binding_type = self.binding_type.map(WorkerBindingType::from);

        Ok(crate::worker_binding::GolemWorkerBinding {
            component_id: self.component_id,
            worker_name,
            idempotency_key,
            response,
            binding_type,
        })
    }
}

impl TryFrom<crate::api_definition::http::HttpApiDefinition> for grpc_apidefinition::ApiDefinition {
    type Error = String;

    fn try_from(
        value: crate::api_definition::http::HttpApiDefinition,
    ) -> Result<Self, Self::Error> {
        let routes = value
            .routes
            .into_iter()
            .map(grpc_apidefinition::HttpRoute::try_from)
            .collect::<Result<Vec<grpc_apidefinition::HttpRoute>, String>>()?;

        let id = value.id.0;

        let definition = grpc_apidefinition::HttpApiDefinition { routes };

        let created_at = prost_types::Timestamp::from(SystemTime::from(value.created_at));

        let result = grpc_apidefinition::ApiDefinition {
            id: Some(grpc_apidefinition::ApiDefinitionId { value: id }),
            version: value.version.0,
            definition: Some(grpc_apidefinition::api_definition::Definition::Http(
                definition,
            )),
            draft: value.draft,
            created_at: Some(created_at),
        };

        Ok(result)
    }
}

impl TryFrom<grpc_apidefinition::ApiDefinition> for crate::api_definition::http::HttpApiDefinition {
    type Error = String;

    fn try_from(value: grpc_apidefinition::ApiDefinition) -> Result<Self, Self::Error> {
        let routes = match value.definition.ok_or("definition is missing")? {
            grpc_apidefinition::api_definition::Definition::Http(http) => http
                .routes
                .into_iter()
                .map(crate::api_definition::http::Route::try_from)
                .collect::<Result<Vec<crate::api_definition::http::Route>, String>>()?,
        };

        let id = value.id.ok_or("Api Definition ID is missing")?;
        let created_at = value
            .created_at
            .ok_or("Created At is missing")
            .and_then(|t| SystemTime::try_from(t).map_err(|_| "Failed to convert timestamp"))?;

        let result = crate::api_definition::http::HttpApiDefinition {
            id: ApiDefinitionId(id.value),
            version: ApiVersion(value.version),
            routes,
            draft: value.draft,
            created_at: created_at.into(),
        };

        Ok(result)
    }
}

impl TryFrom<grpc_apidefinition::v1::ApiDefinitionRequest>
    for crate::api_definition::http::HttpApiDefinitionRequest
{
    type Error = String;

    fn try_from(value: grpc_apidefinition::v1::ApiDefinitionRequest) -> Result<Self, Self::Error> {
        let routes = match value.definition.ok_or("definition is missing")? {
            grpc_apidefinition::v1::api_definition_request::Definition::Http(http) => http
                .routes
                .into_iter()
                .map(crate::api_definition::http::Route::try_from)
                .collect::<Result<Vec<crate::api_definition::http::Route>, String>>()?,
        };

        let id = value.id.ok_or("Api Definition ID is missing")?;

        let result = crate::api_definition::http::HttpApiDefinitionRequest {
            id: ApiDefinitionId(id.value),
            version: ApiVersion(value.version),
            routes,
            draft: value.draft,
        };

        Ok(result)
    }
}

impl TryFrom<crate::api_definition::http::Route> for grpc_apidefinition::HttpRoute {
    type Error = String;

    fn try_from(value: crate::api_definition::http::Route) -> Result<Self, Self::Error> {
        let path = value.path.to_string();
        let binding = grpc_apidefinition::WorkerBinding::try_from(value.binding)?;
        let method: grpc_apidefinition::HttpMethod = value.method.into();

        let result = grpc_apidefinition::HttpRoute {
            method: method as i32,
            path,
            binding: Some(binding),
        };

        Ok(result)
    }
}

impl TryFrom<CompiledRoute> for golem_api_grpc::proto::golem::apidefinition::CompiledHttpRoute {
    type Error = String;

    fn try_from(value: CompiledRoute) -> Result<Self, Self::Error> {
        let method = value.method as i32;
        let path = value.path.to_string();
        let binding = value.binding.try_into()?;
        Ok(Self {
            method,
            path,
            binding: Some(binding),
        })
    }
}

impl TryFrom<golem_api_grpc::proto::golem::apidefinition::CompiledHttpRoute> for CompiledRoute {
    type Error = String;

    fn try_from(
        value: golem_api_grpc::proto::golem::apidefinition::CompiledHttpRoute,
    ) -> Result<Self, Self::Error> {
        let method = MethodPattern::try_from(value.method)?;
        let path = AllPathPatterns::parse(value.path.as_str()).map_err(|e| e.to_string())?;
        let binding = value.binding.ok_or("binding is missing")?.try_into()?;
        Ok(CompiledRoute {
            method,
            path,
            binding,
        })
    }
}

impl From<MethodPattern> for grpc_apidefinition::HttpMethod {
    fn from(value: MethodPattern) -> Self {
        match value {
            MethodPattern::Get => grpc_apidefinition::HttpMethod::Get,
            MethodPattern::Post => grpc_apidefinition::HttpMethod::Post,
            MethodPattern::Put => grpc_apidefinition::HttpMethod::Put,
            MethodPattern::Delete => grpc_apidefinition::HttpMethod::Delete,
            MethodPattern::Patch => grpc_apidefinition::HttpMethod::Patch,
            MethodPattern::Head => grpc_apidefinition::HttpMethod::Head,
            MethodPattern::Options => grpc_apidefinition::HttpMethod::Options,
            MethodPattern::Trace => grpc_apidefinition::HttpMethod::Trace,
            MethodPattern::Connect => grpc_apidefinition::HttpMethod::Connect,
        }
    }
}

impl TryFrom<grpc_apidefinition::HttpRoute> for crate::api_definition::http::Route {
    type Error = String;

    fn try_from(value: grpc_apidefinition::HttpRoute) -> Result<Self, Self::Error> {
        let path = AllPathPatterns::parse(value.path.as_str()).map_err(|e| e.to_string())?;
        let binding = value.binding.ok_or("binding is missing")?.try_into()?;

        let method: MethodPattern = value.method.try_into()?;

        let result = crate::api_definition::http::Route {
            method,
            path,
            binding,
        };

        Ok(result)
    }
}

impl TryFrom<crate::worker_binding::GolemWorkerBinding> for grpc_apidefinition::WorkerBinding {
    type Error = String;

    fn try_from(value: crate::worker_binding::GolemWorkerBinding) -> Result<Self, Self::Error> {
        let response = Some(value.response.0.into());

        let worker_name = Some(value.worker_name.into());

        let idempotency_key = value.idempotency_key.map(|key| key.into());

        let binding_type = value.binding_type
            .map(golem_api_grpc::proto::golem::apidefinition::WorkerBindingType::from)
            .map(|binding_type| binding_type as i32);

        let result = grpc_apidefinition::WorkerBinding {
            component: Some(value.component_id.into()),
            worker_name,
            idempotency_key,
            response,
            binding_type,
        };

        Ok(result)
    }
}

impl TryFrom<grpc_apidefinition::WorkerBinding> for crate::worker_binding::GolemWorkerBinding {
    type Error = String;

    fn try_from(value: grpc_apidefinition::WorkerBinding) -> Result<Self, Self::Error> {
        let binding_type = value.binding_type().into();

        let response: crate::worker_binding::ResponseMapping = {
            let r: Expr = value.response.ok_or("response is missing")?.try_into()?;
            crate::worker_binding::ResponseMapping(r)
        };
        
        let worker_name: Expr = match value.worker_name {
            Some(worker_name) => worker_name.try_into()?,
            None => match binding_type {
                WorkerBindingType::Default => Err("worker name is missing")?,
                // file-server bindings are allowed to be anonymous
                WorkerBindingType::FileServer => Expr::empty_expr(),
            }
        };

        let component_id = value.component.ok_or("component is missing")?.try_into()?;

        let idempotency_key = if let Some(key) = value.idempotency_key {
            Some(key.try_into()?)
        } else {
            None
        };

        let result = crate::worker_binding::GolemWorkerBinding {
            component_id,
            worker_name,
            idempotency_key,
            response,
            binding_type: Some(binding_type),
        };

        Ok(result)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Encode, Decode, Enum)]
pub enum WorkerBindingType {
    #[default]
    Default,
    FileServer,
}

impl std::fmt::Display for WorkerBindingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerBindingType::Default => write!(f, "Default"),
            WorkerBindingType::FileServer => write!(f, "FileServer"),
        }
    }
}

impl From<golem_api_grpc::proto::golem::apidefinition::WorkerBindingType> for WorkerBindingType {
    fn from(value: golem_api_grpc::proto::golem::apidefinition::WorkerBindingType) -> Self {
        match value {
            golem_api_grpc::proto::golem::apidefinition::WorkerBindingType::Default => Self::Default,
            golem_api_grpc::proto::golem::apidefinition::WorkerBindingType::FileServer => Self::FileServer,
        }
    }
}

impl From<WorkerBindingType> for golem_api_grpc::proto::golem::apidefinition::WorkerBindingType {
    fn from(value: WorkerBindingType) -> Self {
        match value {
            WorkerBindingType::Default => Self::Default,
            WorkerBindingType::FileServer => Self::FileServer,
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::api_definition::http::MethodPattern;
    use golem_api_grpc::proto::golem::apidefinition as grpc_apidefinition;
    use test_r::test;

    #[test]
    fn test_method_pattern() {
        for method in 0..8 {
            let method_pattern: MethodPattern = method.try_into().unwrap();
            let method_grpc: grpc_apidefinition::HttpMethod = method_pattern.into();
            assert_eq!(method, method_grpc as i32);
        }
    }
}
