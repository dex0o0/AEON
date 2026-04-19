use std::sync::Arc;
use axum::{Json, Router, extract::State, http::HeaderValue, routing::get};
use serde_json::{json, Value};
use tower_http::cors::{AllowOrigin, CorsLayer};
use crate::monitoring::Systate;
use tokio;



pub async fn start_server(stated:Arc<tokio::sync::Mutex<Systate>>){
    tokio::spawn(async move {
        let cors = CorsLayer::new()
            .allow_origin(
                AllowOrigin::exact(
                    HeaderValue::from_static("http://127.0.0.1:3000")
                )
            )
            .allow_methods([axum::http::Method::GET]);
        let app = Router::new()
            .route("/status", get(status_handle))
            .layer(cors)
            .with_state(stated);

        let lisener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

        axum::serve(lisener,app).await.unwrap();
    });
}


async fn status_handle(State(state):State<Arc<tokio::sync::Mutex<Systate>>>) -> Json<Value>{
    let state = state.lock().await;

    let cpu = *state.cpu_usage.lock().unwrap();
    let mem = *state.mem_useag.lock().unwrap();
    let swap = *state.swap_usage.lock().unwrap();
    
    let disks:Vec<Value> = state.disk.lock().unwrap()
        .iter()
        .map(|d| {
        let total = d.total_space() as f32 / (1024.0 * 1024.0 * 1024.0 );
        let free = d.available_space() as f32 / (1024.0 * 1024.0 * 1024.0 );
        let used = total - free;

        let percent = if total > 0.0 {(used/total) * 100.0} else {0.0};

        json!({
            "name": d.name().to_string_lossy(),
            "mont_point": d.mount_point().to_string_lossy(),
            "total_gb":total,
            "used_gb":used,
            "free_gb":free,
            "useage_percent":percent,
        })
    }).collect();

    Json(json!({
        "cpu":{
            "usage_percent":cpu,
            "status":if cpu > 80.0 {"warning"} else {"normal"},
        },
        "memory":{
            "usage_percent":mem,
            "status":if mem > 80.0 {"warning"} else {"normal"},
        },
        "swap":{
            "usage_percent":swap,
        },
        "disks":*disks,
        "timestamp":chrono::Local::now().to_rfc3339(),
    }))
}
