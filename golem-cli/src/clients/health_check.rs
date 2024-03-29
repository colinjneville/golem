// Copyright 2024 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_trait::async_trait;
use golem_client::model::VersionInfo;
use tracing::info;

use crate::model::GolemError;

#[async_trait]
pub trait HealthCheckClient {
    async fn version(&self) -> Result<VersionInfo, GolemError>;
}

#[derive(Clone)]
pub struct HealthCheckClientLive<C: golem_client::api::HealthCheckClient + Sync + Send> {
    pub client: C,
}

#[async_trait]
impl<C: golem_client::api::HealthCheckClient + Sync + Send> HealthCheckClient for HealthCheckClientLive<C> {
    async fn version(&self) -> Result<VersionInfo, GolemError> {
        info!("Getting version from health check");
        
        Ok(self
            .client
            .version()
            .await?)
    }
}
