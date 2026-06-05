use crate::{
    modules::monitoring::{Icpu, Idisks},
    monitoring::Systate,
};
use axum::{extract::State, http::HeaderValue, routing::get, Json, Router};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub struct AppState {
    pub state: Arc<tokio::sync::Mutex<Systate>>,
    pub idisk: Arc<tokio::sync::Mutex<Idisks>>,
    pub icpu: Arc<tokio::sync::Mutex<Icpu>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            state: Arc::new(tokio::sync::Mutex::new(Systate::new())),
            idisk: Arc::new(tokio::sync::Mutex::new(Idisks::new())),
            icpu: Arc::new(tokio::sync::Mutex::new(Icpu::new())),
        }
    }
}

pub async fn rest_run(app: AppState) {
    start_server(app).await;
    let _ = tokio::time::sleep(Duration::from_secs(1)).await;
}

pub async fn start_server(app: AppState) {
    tokio::spawn(async move {
        //get free port
        //and
        //HeaderValue
        let (free_port, listener) = get_free_port(3000, 200).await;
        let r = format!("http://127.0.0.1:{}", free_port);

        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::exact(
                HeaderValue::from_str(&r)
                    .unwrap_or_else(|_| HeaderValue::from_static("http://127.0.0.1:3000")),
            ))
            .allow_methods([axum::http::Method::GET]);

        let app = Router::new()
            .route("/status", get(status_handle))
            .layer(cors)
            .with_state(Arc::new(app));

        // let lisener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", free_port))
        //     .await
        //     .expect("Failed to created tcp listener");

        //this is for debuging.
        //
        dbg!(free_port, &listener);

        axum::serve(listener, app).await.unwrap();
    });
}

async fn status_handle(State(appstate): State<Arc<AppState>>) -> Json<Value> {
    let icpu = appstate.icpu.lock().await;
    let state = appstate.state.lock().await;
    let idisk = appstate.idisk.lock().await;

    let disks: Vec<Value> = idisk
        .disk
        .lock()
        .unwrap()
        .iter()
        .map(|d| {
            let total = d.total_space() as f32 / (1024.0 * 1024.0 * 1024.0);
            let free = d.available_space() as f32 / (1024.0 * 1024.0 * 1024.0);
            let used = total - free;

            let percent = if total > 0.0 {
                (used / total) * 100.0
            } else {
                0.0
            };

            json!({
                "name": d.name().to_string_lossy(),
                "mont_point": d.mount_point().to_string_lossy(),
                "total_gb":total,
                "used_gb":used,
                "free_gb":free,
                "useage_percent":percent,
            })
        })
        .collect();

    Json(json!({
        "cpu":{
            "usage_percent": *icpu.cpu_usage.lock().unwrap(),
            "status":if *icpu.cpu_usage.lock().unwrap() > 80.0 {"warning"} else {"normal"},
        },
        "memory":{
            "usage_percent": *state.mem_useag.lock().unwrap(),
            "status":if *state.mem_useag.lock().unwrap() >= 80.0 {"warning"} else {"normal"},
        },
        "swap":{
            "usage_percent": *state.swap_usage.lock().unwrap(),
        },
        "disks": *disks,
        "timestamp":chrono::Local::now().to_rfc3339(),
    }))
}

///<h1>---Get_free_Port---</h1>
///<h4>read and found free and free port on your system</h4>
///
///<h3>by example</h3>
///```
///get_free_port(/* s:start_point, t:next_range */);
///
///```
///```
///get_free_port(3000,200);
///```
#[allow(dead_code)]
async fn get_free_port(s: u32, t: u32) -> (u32, tokio::net::TcpListener) {
    for port in s..=(s + t) {
        if let Ok(listener) = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
            return (port, listener);
        }
    }
    panic!("no free port found range 0..65000");
}
