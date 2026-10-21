use std::error::Error;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, json};
use tokio::time::sleep;
use uuid::Uuid;

struct AppHandle {
    child: Child,
    base_url: String,
}

impl Drop for AppHandle {
    fn drop(&mut self) {
        match self.child.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
            Err(_) => {}
        }
    }
}

fn pick_free_port() -> Result<u16, Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

async fn spawn_app() -> Result<AppHandle, Box<dyn Error>> {
    let port = pick_free_port()?;
    let base_url = format!("http://127.0.0.1:{port}");

    let child = Command::new(env!("CARGO_BIN_EXE_vitro-api"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("APP_PORT", port.to_string())
        .env("APP_LOG", "error")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut app = AppHandle { child, base_url };
    let client = Client::new();

    for _ in 0..400 {
        if let Some(status) = app.child.try_wait()? {
            return Err(format!("App exited before ready: {status}").into());
        }

        if let Ok(resp) = client
            .get(format!("{}/health-check", app.base_url))
            .send()
            .await
            && resp.status() == StatusCode::OK
        {
            return Ok(app);
        }

        sleep(Duration::from_millis(150)).await;
    }

    let status_hint = match app.child.try_wait() {
        Ok(Some(status)) => format!(" (process exited: {status})"),
        Ok(None) => String::from(" (process still running)"),
        Err(err) => format!(" (failed to inspect process: {err})"),
    };
    Err(format!("Timed out waiting for app readiness{status_hint}").into())
}

async fn expect_json(
    resp: Response,
    expected: StatusCode,
    step: &str,
) -> Result<Value, Box<dyn Error>> {
    let status = resp.status();
    let body = resp.text().await?;
    if status != expected {
        return Err(format!(
            "Step `{step}` expected status {expected}, got {status}, body: {body}"
        )
        .into());
    }
    if body.is_empty() {
        return Ok(json!({}));
    }
    Ok(serde_json::from_str(&body)?)
}

fn contains_id(items: &[Value], id: &str) -> bool {
    items
        .iter()
        .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
}

#[tokio::test]
async fn api_e2e_non_file_endpoints_work() -> Result<(), Box<dyn Error>> {
    eprintln!("[api_e2e] step: boot app");
    let app = spawn_app().await?;
    let client = Client::new();

    let seed = Uuid::new_v4().to_string().replace('-', "");
    let email = format!("e2e_{seed}@example.com");
    let password = String::from("secret123");
    let name = format!("John Doe | {seed}");

    eprintln!("[api_e2e] step: auth.signup");
    let signup_resp = client
        .post(format!("{}/auth/signup", app.base_url))
        .json(&json!({
            "email": email,
            "password": password,
            "name": name,
        }))
        .send()
        .await?;
    let signup_json = expect_json(signup_resp, StatusCode::CREATED, "auth.signup").await?;

    let workspace_id = signup_json["data"]["user"]["default_workspace_id"]
        .as_str()
        .ok_or("Missing default_workspace_id in signup response")?
        .to_string();

    eprintln!("[api_e2e] step: auth.login");
    let login_resp = client
        .post(format!("{}/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await?;
    let login_json = expect_json(login_resp, StatusCode::OK, "auth.login").await?;
    let access_token = login_json["data"]["access_token"]
        .as_str()
        .ok_or("Missing access_token in login response")?
        .to_string();

    eprintln!(
        "[api_e2e] Test account: \n- Email: {}\n- Password: {}\n- Login details: {}",
        email, password, login_json
    );

    eprintln!("[api_e2e] step: auth.me");
    let me_resp = client
        .get(format!("{}/auth/me", app.base_url))
        .bearer_auth(&access_token)
        .send()
        .await?;
    let me_json = expect_json(me_resp, StatusCode::OK, "auth.me").await?;
    let me_email = me_json["data"]["email"]
        .as_str()
        .ok_or("Missing email in /auth/me response")?;
    assert_eq!(me_email, email);

    eprintln!("[api_e2e] step: auth.exchange_google_placeholder");
    let exchange_resp = client
        .post(format!("{}/auth/exchange", app.base_url))
        .json(&json!({
            "provider": "google",
            "token": "placeholder-token",
        }))
        .send()
        .await?;
    let _ = expect_json(
        exchange_resp,
        StatusCode::NOT_IMPLEMENTED,
        "auth.exchange_google_placeholder",
    )
    .await?;

    let dataset_name = format!("dataset_{seed}");

    eprintln!("[api_e2e] step: datasets.create_invalid_name");
    let create_dataset_invalid_resp = client
        .post(format!("{}/datasets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "name": "Not Snake Case!",
            "display_name": "E2E Dataset Invalid",
            "description": "dataset for api e2e",
            "tags": ["e2e", "dataset"],
        }))
        .send()
        .await?;
    let _ = expect_json(
        create_dataset_invalid_resp,
        StatusCode::BAD_REQUEST,
        "datasets.create_invalid_name",
    )
    .await?;

    eprintln!("[api_e2e] step: datasets.create");
    let create_dataset_resp = client
        .post(format!("{}/datasets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "name": dataset_name,
            "display_name": "E2E Dataset",
            "description": "dataset for api e2e",
            "tags": ["e2e", "dataset"],
        }))
        .send()
        .await?;
    let created_dataset =
        expect_json(create_dataset_resp, StatusCode::CREATED, "datasets.create").await?;
    let dataset_id = created_dataset["data"]["id"]
        .as_str()
        .ok_or("Missing dataset id")?
        .to_string();

    eprintln!("[api_e2e] step: datasets.list");
    let list_datasets_resp = client
        .get(format!("{}/datasets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .query(&[("page", "1"), ("page_size", "20")])
        .send()
        .await?;
    let list_datasets_json =
        expect_json(list_datasets_resp, StatusCode::OK, "datasets.list").await?;
    let datasets = list_datasets_json["data"]
        .as_array()
        .ok_or("Expected datasets data array")?;
    assert!(
        contains_id(datasets, &dataset_id),
        "Step `datasets.list`: created dataset id {dataset_id} not found in list response"
    );

    eprintln!("[api_e2e] step: datasets.get");
    let get_dataset_resp = client
        .get(format!("{}/datasets/{}", app.base_url, dataset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(get_dataset_resp, StatusCode::OK, "datasets.get").await?;

    eprintln!("[api_e2e] step: datasets.update");
    let update_dataset_resp = client
        .patch(format!("{}/datasets/{}", app.base_url, dataset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "display_name": "E2E Dataset Updated",
            "description": "updated",
            "tags": ["e2e", "updated"],
        }))
        .send()
        .await?;
    let _ = expect_json(update_dataset_resp, StatusCode::OK, "datasets.update").await?;

    let mut asset_ids = Vec::new();
    for idx in 1..=3 {
        eprintln!("[api_e2e] step: assets.create.{idx}");
        let create_asset_resp = client
            .post(format!("{}/assets", app.base_url))
            .bearer_auth(&access_token)
            .header("X-Workspace-Id", &workspace_id)
            .json(&json!({
                "dataset_id": dataset_id,
                "name": format!("asset_{}_{}", seed, idx),
                "display_name": format!("Asset {}", idx),
                "description": format!("asset {}", idx),
                "tags": ["e2e", format!("idx{idx}")],
            }))
            .send()
            .await?;
        let asset_json = expect_json(
            create_asset_resp,
            StatusCode::CREATED,
            &format!("assets.create.{idx}"),
        )
        .await?;
        let asset_id = asset_json["data"]["id"]
            .as_str()
            .ok_or("Missing asset id")?
            .to_string();
        asset_ids.push(asset_id);
    }

    eprintln!("[api_e2e] step: datasets.create_second");
    let second_dataset_name = format!("dataset_{seed}_second");
    let create_second_dataset_resp = client
        .post(format!("{}/datasets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "name": second_dataset_name,
            "display_name": "E2E Dataset Second",
            "description": "second dataset for api e2e",
            "tags": ["e2e", "dataset"],
        }))
        .send()
        .await?;
    let created_second_dataset = expect_json(
        create_second_dataset_resp,
        StatusCode::CREATED,
        "datasets.create_second",
    )
    .await?;
    let second_dataset_id = created_second_dataset["data"]["id"]
        .as_str()
        .ok_or("Missing dataset id")?
        .to_string();

    eprintln!("[api_e2e] step: assets.create_second_dataset_asset");
    let create_second_asset_resp = client
        .post(format!("{}/assets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "dataset_id": second_dataset_id,
            "name": format!("asset_{}_second", seed),
            "display_name": "Asset Second",
            "description": "asset in second dataset",
            "tags": ["e2e", "second"],
        }))
        .send()
        .await?;
    let second_asset_json = expect_json(
        create_second_asset_resp,
        StatusCode::CREATED,
        "assets.create_second_dataset_asset",
    )
    .await?;
    let second_asset_id = second_asset_json["data"]["id"]
        .as_str()
        .ok_or("Missing asset id")?
        .to_string();

    eprintln!("[api_e2e] step: assets.list");
    let list_assets_resp = client
        .get(format!("{}/assets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .query(&[
            ("page", "1"),
            ("page_size", "20"),
            ("dataset_id", dataset_id.as_str()),
        ])
        .send()
        .await?;
    let list_assets_json = expect_json(list_assets_resp, StatusCode::OK, "assets.list").await?;
    let assets = list_assets_json["data"]
        .as_array()
        .ok_or("Expected assets data array")?;
    for asset_id in &asset_ids {
        assert!(
            contains_id(assets, asset_id),
            "Step `assets.list`: created asset id {asset_id} not found in list response"
        );
    }
    assert!(
        !contains_id(assets, &second_asset_id),
        "Step `assets.list`: filtered list unexpectedly contained second-dataset asset id {second_asset_id}"
    );

    eprintln!("[api_e2e] step: assets.list_without_dataset_id");
    let list_assets_no_filter_resp = client
        .get(format!("{}/assets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .query(&[("page", "1"), ("page_size", "20")])
        .send()
        .await?;
    let list_assets_no_filter_json = expect_json(
        list_assets_no_filter_resp,
        StatusCode::OK,
        "assets.list_without_dataset_id",
    )
    .await?;
    let assets_no_filter = list_assets_no_filter_json["data"]
        .as_array()
        .ok_or("Expected assets data array")?;
    for asset_id in &asset_ids {
        assert!(
            contains_id(assets_no_filter, asset_id),
            "Step `assets.list_without_dataset_id`: created asset id {asset_id} not found in list response"
        );
    }
    assert!(
        contains_id(assets_no_filter, &second_asset_id),
        "Step `assets.list_without_dataset_id`: second-dataset asset id {second_asset_id} not found in cross-dataset list response"
    );

    eprintln!("[api_e2e] step: assets.get");
    let get_asset_resp = client
        .get(format!("{}/assets/{}", app.base_url, asset_ids[0]))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(get_asset_resp, StatusCode::OK, "assets.get").await?;

    eprintln!("[api_e2e] step: assets.update");
    let update_asset_resp = client
        .patch(format!("{}/assets/{}", app.base_url, asset_ids[0]))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "display_name": "Asset Updated",
            "description": "asset updated",
            "tags": ["e2e", "updated"],
        }))
        .send()
        .await?;
    let _ = expect_json(update_asset_resp, StatusCode::OK, "assets.update").await?;

    eprintln!("[api_e2e] step: tasks.create");
    let create_task_resp = client
        .post(format!("{}/tasks", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "asset_id": asset_ids[1],
            "task_type": "FULL_ANALYSIS",
            "configs": {
                "mode": "smoke"
            }
        }))
        .send()
        .await?;
    let task_json = expect_json(create_task_resp, StatusCode::CREATED, "tasks.create").await?;
    let task_id = task_json["data"]["id"]
        .as_str()
        .ok_or("Missing task id")?
        .to_string();

    eprintln!("[api_e2e] step: tasks.list");
    let list_tasks_resp = client
        .get(format!("{}/tasks", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .query(&[
            ("page", "1"),
            ("page_size", "20"),
            ("asset_id", asset_ids[1].as_str()),
        ])
        .send()
        .await?;
    let list_tasks_json = expect_json(list_tasks_resp, StatusCode::OK, "tasks.list").await?;
    let tasks = list_tasks_json["data"]
        .as_array()
        .ok_or("Expected tasks data array")?;
    assert!(
        contains_id(tasks, &task_id),
        "Step `tasks.list`: created task id {task_id} not found in list response"
    );

    eprintln!("[api_e2e] step: tasks.get");
    let get_task_resp = client
        .get(format!("{}/tasks/{}", app.base_url, task_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(get_task_resp, StatusCode::OK, "tasks.get").await?;

    eprintln!("[api_e2e] step: task_runs.list_by_task");
    let list_task_runs_resp = client
        .get(format!("{}/tasks/{}/runs", app.base_url, task_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .query(&[("page", "1"), ("page_size", "20")])
        .send()
        .await?;
    let _ = expect_json(
        list_task_runs_resp,
        StatusCode::OK,
        "task_runs.list_by_task",
    )
    .await?;

    eprintln!("[api_e2e] step: task_runs.get_missing");
    let get_task_run_resp = client
        .get(format!("{}/task-runs/{}", app.base_url, Uuid::new_v4()))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(
        get_task_run_resp,
        StatusCode::NOT_FOUND,
        "task_runs.get_missing",
    )
    .await?;

    eprintln!("[api_e2e] step: assets.archive_once");
    let archive_asset_resp = client
        .delete(format!("{}/assets/{}", app.base_url, asset_ids[2]))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(archive_asset_resp, StatusCode::OK, "assets.archive_once").await?;

    eprintln!("[api_e2e] step: assets.archive_again");
    let archive_asset_again_resp = client
        .delete(format!("{}/assets/{}", app.base_url, asset_ids[2]))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(
        archive_asset_again_resp,
        StatusCode::GONE,
        "assets.archive_again",
    )
    .await?;

    eprintln!("[api_e2e] step: datasets.archive_once");
    let archive_dataset_resp = client
        .delete(format!("{}/datasets/{}", app.base_url, dataset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(
        archive_dataset_resp,
        StatusCode::OK,
        "datasets.archive_once",
    )
    .await?;

    eprintln!("[api_e2e] step: datasets.archive_again");
    let archive_dataset_again_resp = client
        .delete(format!("{}/datasets/{}", app.base_url, dataset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(
        archive_dataset_again_resp,
        StatusCode::GONE,
        "datasets.archive_again",
    )
    .await?;

    Ok(())
}

async fn expect_bytes(
    resp: Response,
    expected: StatusCode,
    step: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let status = resp.status();
    if status != expected {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Step `{step}` expected status {expected}, got {status}, body: {body}"
        )
        .into());
    }
    Ok(resp.bytes().await?.to_vec())
}

#[tokio::test]
async fn api_e2e_file_endpoints_work() -> Result<(), Box<dyn Error>> {
    eprintln!("[api_e2e_files] step: boot app");
    let app = spawn_app().await?;
    let client = Client::new();

    let seed = Uuid::new_v4().to_string().replace('-', "");
    let email = format!("e2e_files_{seed}@example.com");
    let password = String::from("secret123");
    let name = format!("Jane Files | {seed}");

    eprintln!("[api_e2e_files] step: auth.signup");
    let signup_resp = client
        .post(format!("{}/auth/signup", app.base_url))
        .json(&json!({
            "email": email,
            "password": password,
            "name": name,
        }))
        .send()
        .await?;
    let signup_json = expect_json(signup_resp, StatusCode::CREATED, "auth.signup").await?;
    let workspace_id = signup_json["data"]["user"]["default_workspace_id"]
        .as_str()
        .ok_or("Missing default_workspace_id in signup response")?
        .to_string();

    eprintln!("[api_e2e_files] step: auth.login");
    let login_resp = client
        .post(format!("{}/auth/login", app.base_url))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await?;
    let login_json = expect_json(login_resp, StatusCode::OK, "auth.login").await?;
    let access_token = login_json["data"]["access_token"]
        .as_str()
        .ok_or("Missing access_token in login response")?
        .to_string();

    let dataset_name = format!("dataset_files_{seed}");
    eprintln!("[api_e2e_files] step: datasets.create");
    let create_dataset_resp = client
        .post(format!("{}/datasets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "name": dataset_name,
            "display_name": "E2E Files Dataset",
            "description": "dataset for api e2e file tests",
            "tags": ["e2e", "files"],
        }))
        .send()
        .await?;
    let created_dataset =
        expect_json(create_dataset_resp, StatusCode::CREATED, "datasets.create").await?;
    let dataset_id = created_dataset["data"]["id"]
        .as_str()
        .ok_or("Missing dataset id")?
        .to_string();

    eprintln!("[api_e2e_files] step: assets.create");
    let create_asset_resp = client
        .post(format!("{}/assets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "dataset_id": dataset_id,
            "name": format!("asset_files_{seed}"),
            "display_name": "Asset Files",
            "description": "asset for file e2e",
            "tags": ["e2e", "files"],
        }))
        .send()
        .await?;
    let asset_json = expect_json(create_asset_resp, StatusCode::CREATED, "assets.create").await?;
    let asset_id = asset_json["data"]["id"]
        .as_str()
        .ok_or("Missing asset id")?
        .to_string();

    eprintln!("[api_e2e_files] step: assets.create_second (for cross-asset download check)");
    let create_other_asset_resp = client
        .post(format!("{}/assets", app.base_url))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .json(&json!({
            "dataset_id": dataset_id,
            "name": format!("asset_files_other_{seed}"),
            "display_name": "Asset Files Other",
            "description": "second asset for cross-asset download check",
            "tags": ["e2e", "files"],
        }))
        .send()
        .await?;
    let other_asset_json = expect_json(
        create_other_asset_resp,
        StatusCode::CREATED,
        "assets.create_second",
    )
    .await?;
    let other_asset_id = other_asset_json["data"]["id"]
        .as_str()
        .ok_or("Missing other asset id")?
        .to_string();

    let file_name = "e2e_upload.csv";
    let file_bytes: Vec<u8> = b"col_a,col_b,col_c\n1,2,3\n4,5,6\n".to_vec();

    eprintln!("[api_e2e_files] step: files.upload");
    let upload_form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(file_bytes.clone())
            .file_name(file_name)
            .mime_str("text/csv")?,
    );
    let upload_resp = client
        .post(format!("{}/assets/{}/upload", app.base_url, asset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .multipart(upload_form)
        .send()
        .await?;
    let upload_json = expect_json(upload_resp, StatusCode::OK, "files.upload").await?;
    let uploaded_file_id = upload_json["data"]["id"]
        .as_str()
        .ok_or("Missing file id in upload response")?
        .to_string();
    assert_eq!(
        upload_json["data"]["file_name"].as_str(),
        Some(file_name),
        "Step `files.upload`: unexpected file_name in response"
    );
    assert_eq!(
        upload_json["data"]["size_bytes"].as_u64(),
        Some(file_bytes.len() as u64),
        "Step `files.upload`: unexpected size_bytes in response"
    );
    assert_eq!(
        upload_json["data"]["asset_id"].as_str(),
        Some(asset_id.as_str()),
        "Step `files.upload`: unexpected asset_id in response"
    );
    assert!(
        upload_json["data"]["content_type"]
            .as_str()
            .is_some_and(|ct| ct.contains("csv")),
        "Step `files.upload`: expected a csv content_type, got {:?}",
        upload_json["data"]["content_type"]
    );
    assert!(
        upload_json["data"]["created_at"].as_str().is_some(),
        "Step `files.upload`: missing created_at in response"
    );

    eprintln!("[api_e2e_files] step: files.list");
    let list_files_resp = client
        .get(format!("{}/assets/{}/files", app.base_url, asset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let list_files_json = expect_json(list_files_resp, StatusCode::OK, "files.list").await?;
    let files = list_files_json["data"]
        .as_array()
        .ok_or("Expected files data array")?;
    let listed_file = files
        .iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(uploaded_file_id.as_str()))
        .ok_or_else(|| {
            format!(
                "Step `files.list`: uploaded file id {uploaded_file_id} not found in list response"
            )
        })?;
    assert_eq!(
        listed_file["file_name"].as_str(),
        Some(file_name),
        "Step `files.list`: unexpected file_name in list entry"
    );
    assert_eq!(
        listed_file["size_bytes"].as_u64(),
        Some(file_bytes.len() as u64),
        "Step `files.list`: unexpected size_bytes in list entry"
    );

    eprintln!("[api_e2e_files] step: files.download");
    let download_resp = client
        .get(format!(
            "{}/assets/{}/files/{}/download",
            app.base_url, asset_id, uploaded_file_id
        ))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let downloaded_bytes = expect_bytes(download_resp, StatusCode::OK, "files.download").await?;
    assert_eq!(
        downloaded_bytes, file_bytes,
        "Step `files.download`: downloaded bytes do not match the uploaded content"
    );

    eprintln!("[api_e2e_files] step: files.upload_duplicate_name");
    let duplicate_file_bytes: Vec<u8> = b"col_a,col_b,col_c\n9,9,9\n".to_vec();
    let duplicate_upload_form = reqwest::multipart::Form::new().part(
        "file",
        // Same file_name as the first upload, different content: this must
        // not overwrite the first upload's object in storage, nor collide
        // in the DB - both files must remain independently downloadable
        // with their own distinct content.
        reqwest::multipart::Part::bytes(duplicate_file_bytes.clone())
            .file_name(file_name)
            .mime_str("text/csv")?,
    );
    let duplicate_upload_resp = client
        .post(format!("{}/assets/{}/upload", app.base_url, asset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .multipart(duplicate_upload_form)
        .send()
        .await?;
    let duplicate_upload_json = expect_json(
        duplicate_upload_resp,
        StatusCode::OK,
        "files.upload_duplicate_name",
    )
    .await?;
    let duplicate_file_id = duplicate_upload_json["data"]["id"]
        .as_str()
        .ok_or("Missing file id in duplicate upload response")?
        .to_string();
    assert_ne!(
        duplicate_file_id, uploaded_file_id,
        "Step `files.upload_duplicate_name`: duplicate-name upload must get a distinct file id"
    );

    eprintln!("[api_e2e_files] step: files.download_first_after_duplicate");
    let download_first_resp = client
        .get(format!(
            "{}/assets/{}/files/{}/download",
            app.base_url, asset_id, uploaded_file_id
        ))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let downloaded_first_bytes = expect_bytes(
        download_first_resp,
        StatusCode::OK,
        "files.download_first_after_duplicate",
    )
    .await?;
    assert_eq!(
        downloaded_first_bytes, file_bytes,
        "Step `files.download_first_after_duplicate`: first upload's content must be unaffected by the same-name second upload"
    );

    eprintln!("[api_e2e_files] step: files.download_duplicate");
    let download_duplicate_resp = client
        .get(format!(
            "{}/assets/{}/files/{}/download",
            app.base_url, asset_id, duplicate_file_id
        ))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let downloaded_duplicate_bytes = expect_bytes(
        download_duplicate_resp,
        StatusCode::OK,
        "files.download_duplicate",
    )
    .await?;
    assert_eq!(
        downloaded_duplicate_bytes, duplicate_file_bytes,
        "Step `files.download_duplicate`: second upload's content must be independently downloadable"
    );

    eprintln!("[api_e2e_files] step: files.download_wrong_asset");
    let download_wrong_asset_resp = client
        .get(format!(
            "{}/assets/{}/files/{}/download",
            app.base_url, other_asset_id, uploaded_file_id
        ))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(
        download_wrong_asset_resp,
        StatusCode::NOT_FOUND,
        "files.download_wrong_asset",
    )
    .await?;

    eprintln!("[api_e2e_files] step: assets.archive_for_upload_regression");
    let archive_asset_resp = client
        .delete(format!("{}/assets/{}", app.base_url, asset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .send()
        .await?;
    let _ = expect_json(
        archive_asset_resp,
        StatusCode::OK,
        "assets.archive_for_upload_regression",
    )
    .await?;

    eprintln!("[api_e2e_files] step: files.upload_to_archived_asset_rejected");
    let upload_to_archived_form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(file_bytes.clone())
            .file_name(file_name)
            .mime_str("text/csv")?,
    );
    let upload_to_archived_resp = client
        .post(format!("{}/assets/{}/upload", app.base_url, asset_id))
        .bearer_auth(&access_token)
        .header("X-Workspace-Id", &workspace_id)
        .multipart(upload_to_archived_form)
        .send()
        .await?;
    let _ = expect_json(
        upload_to_archived_resp,
        StatusCode::GONE,
        "files.upload_to_archived_asset_rejected",
    )
    .await?;

    Ok(())
}
