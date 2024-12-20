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

use golem_common::model::ComponentId;
use test_r::{add_test, inherit_test_dep, test_dep, test_gen};
use tracing::debug;

use crate::cli::{Cli, CliLive};
use crate::worker::{add_component_from_file, add_component_from_file_with_manifest};
use crate::Tracing;
use assert2::assert;
use chrono::{DateTime, Utc};
use golem_cli::model::component::ComponentView;
use golem_client::model::{
    GolemWorkerBinding, GolemWorkerBindingWithTypeInfo, HttpApiDefinitionRequest,
    HttpApiDefinitionWithTypeInfo, MethodPattern, RibInputTypeInfo, Route, RouteWithTypeInfo,
    VersionedComponentId, WorkerBindingType,
};
use golem_test_framework::config::{EnvBasedTestDependencies, TestDependencies};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use test_r::core::{DynamicTestRegistration, TestType};
use uuid::Uuid;

inherit_test_dep!(EnvBasedTestDependencies);
inherit_test_dep!(Tracing);

#[test_dep]
fn cli(deps: &EnvBasedTestDependencies) -> CliLive {
    CliLive::make("api_definition", Arc::new(deps.clone())).unwrap()
}

#[test_gen]
fn generated(r: &mut DynamicTestRegistration) {
    make(r, "_short", "CLI_short", true);
    make(r, "_long", "CLI_long", false);
}

fn make(r: &mut DynamicTestRegistration, suffix: &'static str, name: &'static str, short: bool) {
    add_test!(
        r,
        format!("api_definition_import{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_import((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_add{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_add((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_update{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_update((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_update_immutable{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_update_immutable((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_list{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_list((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_list_versions{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_list_versions((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_get{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_get((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_delete{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_delete((deps, name.to_string(), cli.with_args(short)))
        }
    );
    add_test!(
        r,
        format!("api_definition_add_file_server{suffix}"),
        TestType::IntegrationTest,
        move |deps: &EnvBasedTestDependencies, cli: &CliLive, _tracing: &Tracing| {
            api_definition_add_file_server((deps, name.to_string(), cli.with_args(short)))
        }
    );
}

pub fn make_shopping_cart_component(
    deps: &(impl TestDependencies + Send + Sync + 'static),
    component_name: &str,
    cli: &CliLive,
) -> anyhow::Result<ComponentView> {
    add_component_from_file(deps, component_name, cli, "shopping-cart.wasm")
}

fn make_file(id: &str, json: &serde_json::value::Value) -> anyhow::Result<PathBuf> {
    let text = serde_json::to_string_pretty(json)?;

    let path = PathBuf::from(format!("../target/api-definition-{id}.json"));

    fs::write(&path, text)?;

    Ok(path)
}

fn golem_def_with_response(
    id: &str,
    component_id: &str,
    response: String,
) -> HttpApiDefinitionRequest {
    HttpApiDefinitionRequest {
        id: id.to_string(),
        version: "0.1.0".to_string(),
        draft: true,
        routes: vec![Route {
            method: MethodPattern::Get,
            path: "/{user-id}/get-cart-contents".to_string(),
            binding: GolemWorkerBinding {
                component_id: VersionedComponentId {
                    component_id: Uuid::parse_str(component_id).unwrap(),
                    version: 0,
                },
                worker_name: "\"foo\"".to_string(),
                idempotency_key: None,
                response,
                binding_type: WorkerBindingType::Default,
            },
        }],
    }
}

pub fn golem_def(id: &str, component_id: &str) -> HttpApiDefinitionRequest {
    golem_def_with_response(
        id,
        component_id,
        "let x = golem:it/api.{checkout}();\nlet status: u64 = 200;\n{headers: {ContentType: \"json\", userid: \"foo\"}, body: \"foo\", status: status}"
            .to_string(),
    )
}

pub fn make_golem_file(def: &HttpApiDefinitionRequest) -> anyhow::Result<PathBuf> {
    let golem_json = serde_json::to_value(def)?;

    make_file(&def.id, &golem_json)
}

pub fn make_open_api_file(
    id: &str,
    component_id: &str,
    component_version: u64,
) -> anyhow::Result<PathBuf> {
    let open_api_json = json!(
      {
        "openapi": "3.0.0",
        "info": {
          "title": "Sample API",
          "version": "1.0.2"
        },
        "x-golem-api-definition-id": id,
        "x-golem-api-definition-version": "0.1.0",
        "paths": {
            "/{user-id}/get-cart-contents": {
              "x-golem-worker-bridge": {
                "worker-name": "\"foo\"",
                "component-id": component_id,
                "component-version": component_version,
                "response" : "let x = golem:it/api.{checkout}();\nlet status: u64 = 200; {headers : {ContentType: \"json\", userid: \"foo\"}, body: \"foo\", status: status}"
              },
              "get": {
                "summary": "Get Cart Contents",
                "description": "Get the contents of a user's cart",
                "parameters": [
                  {
                    "name": "user-id",
                    "in": "path",
                    "required": true,
                    "schema": {
                      "type": "string"
                    }
                  }
                ],
                "responses": {
                  "200": {
                    "description": "OK",
                    "content":{
                      "application/json": {
                        "schema": {
                          "$ref": "#/components/schemas/CartItem"
                        }
                      }

                    }
                  },
                  "404": {
                    "description": "Contents not found"
                  }
                }
              }
            }
          },
          "components": {
            "schemas": {
              "CartItem": {
                "type": "object",
                "properties": {
                  "id": {
                    "type": "string"
                  },
                  "name": {
                    "type": "string"
                  },
                  "price": {
                    "type": "number"
                  }
                }
              }
            }
        }
        }
    );

    make_file(id, &open_api_json)
}

pub fn to_definition(
    request: HttpApiDefinitionRequest,
    created_at: Option<DateTime<Utc>>,
) -> HttpApiDefinitionWithTypeInfo {
    HttpApiDefinitionWithTypeInfo {
        id: request.id,
        version: request.version,
        draft: request.draft,
        routes: request
            .routes
            .iter()
            .map(|v0| {
                let v = v0.clone();

                RouteWithTypeInfo {
                    method: v.method,
                    path: v.path,
                    binding: GolemWorkerBindingWithTypeInfo {
                        component_id: v.binding.component_id,
                        worker_name: v.binding.worker_name.clone(),
                        idempotency_key: v.binding.idempotency_key.clone(),
                        response: v.binding.response,
                        binding_type: v.binding.binding_type.clone(),
                        response_mapping_input: Some(RibInputTypeInfo {
                            types: HashMap::new(),
                        }),
                        worker_name_input: Some(RibInputTypeInfo {
                            types: HashMap::new(),
                        }),
                        idempotency_key_input: None,
                    },
                }
            })
            .collect(),
        created_at,
    }
}

fn api_definition_import(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_import{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();
    let component_version = component.component_version;
    let path = make_open_api_file(&component_name, &component_id, component_version)?;

    let res: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "import", path.to_str().unwrap()])?;

    let expected = to_definition(golem_def(&component_name, &component_id), res.created_at);

    assert_eq!(res, expected);

    Ok(())
}

fn api_definition_add(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_add{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();
    let def = golem_def(&component_name, &component_id);
    let path = make_golem_file(&def)?;

    let res: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let expected = to_definition(def, res.created_at);
    assert_eq!(res, expected);

    Ok(())
}

fn api_definition_update(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_update{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();

    let def = golem_def(&component_name, &component_id);
    let path = make_golem_file(&def)?;
    let _: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let updated = golem_def_with_response(
        &component_name,
        &component_id,
        "let status: u64 = 200;\n{headers: {ContentType: \"json\", userid: \"bar\"}, body: \"baz\", status: status}"
            .to_string(),
    );
    let path = make_golem_file(&updated)?;
    let res: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "update", path.to_str().unwrap()])?;

    let expected = to_definition(updated, res.created_at);

    assert_eq!(res, expected);

    Ok(())
}

fn api_definition_update_immutable(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_update_immutable{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();

    let mut def = golem_def(&component_name, &component_id);
    def.draft = false;
    let path = make_golem_file(&def)?;
    let _: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let updated = golem_def_with_response(&component_name, &component_id, "${let status: u64 = 200; {headers: {ContentType: \"json\", userid: \"bar\"}, body: worker.response, status: status}}".to_string());
    let path = make_golem_file(&updated)?;
    let res = cli.run_string(&["api-definition", "update", path.to_str().unwrap()]);

    assert!(res.is_err());

    Ok(())
}

fn api_definition_list(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_list{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();
    let def = golem_def(&component_name, &component_id);
    let path = make_golem_file(&def)?;

    let _: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let res: Vec<HttpApiDefinitionWithTypeInfo> = cli.run(&["api-definition", "list"])?;

    let found = res.into_iter().find(|d| {
        let e = to_definition(def.clone(), d.created_at);
        d == &e
    });

    assert!(found.is_some());

    Ok(())
}

fn api_definition_list_versions(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_list_versions{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();
    let def = golem_def(&component_name, &component_id);
    let path = make_golem_file(&def)?;
    let cfg = &cli.config;

    let _: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let res: Vec<HttpApiDefinitionWithTypeInfo> = cli.run(&[
        "api-definition",
        "list",
        &cfg.arg('i', "id"),
        &component_name,
    ])?;

    assert_eq!(res.len(), 1);

    let res: HttpApiDefinitionWithTypeInfo = res.first().unwrap().clone();
    let expected = to_definition(def, res.created_at);

    assert_eq!(res, expected);

    Ok(())
}

fn api_definition_get(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_get{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();
    let def = golem_def(&component_name, &component_id);
    let path = make_golem_file(&def)?;

    let _: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let cfg = &cli.config;

    let res: HttpApiDefinitionWithTypeInfo = cli.run(&[
        "api-definition",
        "get",
        &cfg.arg('i', "id"),
        &component_name,
        &cfg.arg('V', "version"),
        "0.1.0",
    ])?;

    let expected = to_definition(def, res.created_at);

    assert_eq!(res, expected);

    Ok(())
}

fn api_definition_delete(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    let component_name = format!("api_definition_delete{name}");
    let component = make_shopping_cart_component(deps, &component_name, &cli)?;
    let component_id = component.component_urn.id.0.to_string();
    let def = golem_def(&component_name, &component_id);
    let path = make_golem_file(&def)?;

    let _: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", path.to_str().unwrap()])?;

    let cfg = &cli.config;

    let res: HttpApiDefinitionWithTypeInfo = cli.run(&[
        "api-definition",
        "get",
        &cfg.arg('i', "id"),
        &component_name,
        &cfg.arg('V', "version"),
        "0.1.0",
    ])?;

    let expected = to_definition(def, res.created_at);

    assert_eq!(res, expected);

    cli.run_unit(&[
        "api-definition",
        "delete",
        &cfg.arg('i', "id"),
        &component_name,
        &cfg.arg('V', "version"),
        "0.1.0",
    ])?;

    let res_list: Vec<HttpApiDefinitionWithTypeInfo> = cli.run(&[
        "api-definition",
        "list",
        &cfg.arg('i', "id"),
        &component_name,
    ])?;

    assert!(res_list.is_empty());

    Ok(())
}

fn api_definition_add_file_server(
    (deps, name, cli): (
        &(impl TestDependencies + Send + Sync + 'static),
        String,
        CliLive,
    ),
) -> anyhow::Result<()> {
    

    let component_name = format!("api_definition_add_file_server{name}");

    let component_dir = prepare_component_directory(deps, &component_name)?;
    let manifest_path = component_dir.path().join(format!("{component_name}.yaml"));

    let component = add_component_from_file_with_manifest(deps, &component_name, &cli, "file-initial.wasm", manifest_path.to_string_lossy().as_ref())?;
    let component_id = component.component_urn.id;

    let api_name = format!("{component_name}-file-server");
    let api_definition = make_file_server_api_definition(&component_id, &api_name)?;
    let api_path = component_dir.path().join("api-file-server.json");

    {
        let mut api_file = std::fs::File::create_new(&api_path)?;
        serde_json::to_writer(&mut api_file, &api_definition)?;
        api_file.flush()?;
    }

    let mut res: HttpApiDefinitionWithTypeInfo =
        cli.run(&["api-definition", "add", api_path.to_str().unwrap()])?;

    let expected = to_definition(api_definition, res.created_at);
    res.routes[0].binding.response_mapping_input.as_mut().unwrap().types.clear();
    assert_eq!(res, expected);

    Ok(())
}

fn copy_dir(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let walk = walkdir::WalkDir::new(source);

    for entry in walk {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            let source_path = entry.path();
            let destination_path = destination.join(source_path.strip_prefix(source)?);
            debug!("Copying '{}' to '{}'", source_path.display(), destination_path.display());

            if let Some(dirs) = destination_path.parent() {
                std::fs::create_dir_all(dirs)?;
            }
            std::fs::copy(source_path, destination_path)?;
        }
    }

    Ok(())
}

fn prepare_component_directory(deps: &(impl TestDependencies + Send + Sync + 'static), component_name: &str) -> anyhow::Result<tempfile::TempDir> {
    let tempdir = tempfile::TempDir::new().unwrap();
    let temp_path = tempdir.path();
    copy_dir(&deps.component_directory().join(Path::new("file-initial/resources/")), temp_path)?;

    let mut application = golem_cli::model::oam::Application::from_yaml_file(temp_path.join("file-initial.yaml"))?;
    let component = application
        .spec
        .components
        .iter_mut()
        .filter(|c| &c.name == "file-initial-1")
        .next()
        .ok_or_else(|| anyhow::anyhow!("Component 'file-initial-1' not found"))?;

    component.name = component_name.to_string();

    let manifest_path = temp_path.join(format!("{component_name}.yaml"));
    application.to_yaml_file(&manifest_path)?;

    Ok(tempdir)
}

fn make_file_server_api_definition(component_id: &ComponentId, api_id: &str) -> anyhow::Result<HttpApiDefinitionRequest> {
    Ok(serde_yaml::from_str(&format!(r#"
        id: '{api_id}'
        version: "0.1.0"
        draft: true
        routes:
        - method: Get
          path: '/static/{{file}}'
          binding:
            componentId: 
              version: 0
              componentId: '{component_id}'
            workerName: '"my_file_server"'
            response: 'let file: str = request.path.file; "/static/ro/${{file}}"'
            bindingType: FileServer
    "#,))?)
}