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
use wasmtime::component::Resource;

use crate::durable_host::DurableWorkerCtx;
use crate::metrics::wasm::record_host_function_call;
use crate::preview2::wasi::keyvalue::atomic::{Bucket, Error, Host, Key};
use crate::workerctx::WorkerCtx;

#[async_trait]
impl<Ctx: WorkerCtx> Host for DurableWorkerCtx<Ctx> {
    async fn increment(
        &mut self,
        _bucket: Resource<Bucket>,
        _key: Key,
        _delta: u64,
    ) -> anyhow::Result<Result<u64, Resource<Error>>> {
        let _permit = self.begin_async_host_function().await?;
        record_host_function_call("keyvalue::atomic", "increment");
        unimplemented!("increment")
    }

    async fn compare_and_swap(
        &mut self,
        _bucket: Resource<Bucket>,
        _key: Key,
        _old: u64,
        _new: u64,
    ) -> anyhow::Result<Result<bool, Resource<Error>>> {
        let _permit = self.begin_async_host_function().await?;
        record_host_function_call("keyvalue::atomic", "compare_and_swap");
        unimplemented!("compare_and_swap")
    }
}

#[async_trait]
impl<Ctx: WorkerCtx> Host for &mut DurableWorkerCtx<Ctx> {
    async fn increment(
        &mut self,
        bucket: Resource<Bucket>,
        key: Key,
        delta: u64,
    ) -> anyhow::Result<Result<u64, Resource<Error>>> {
        (*self).increment(bucket, key, delta).await
    }

    async fn compare_and_swap(
        &mut self,
        bucket: Resource<Bucket>,
        key: Key,
        old: u64,
        new: u64,
    ) -> anyhow::Result<Result<bool, Resource<Error>>> {
        (*self).compare_and_swap(bucket, key, old, new).await
    }
}
