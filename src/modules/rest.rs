use crate::monitoring::Systate;
use axum::{extract::State, http::HeaderValue, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub async fn start_server(stated: Arc<tokio::sync::Mutex<Systate>>) {
    tokio::spawn(async move {
        //get free port
        //and
        //HeaderValue
        let free_port = get_open_port(3000, 200).await;
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
            .with_state(stated);

        let lisener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", free_port))
            .await
            .expect("Failed to created tcp listener");

        //this is for debuging.
        //
        dbg!(free_port, &lisener);

        axum::serve(lisener, app).await.unwrap();
    });
}

async fn status_handle(State(state): State<Arc<tokio::sync::Mutex<Systate>>>) -> Json<Value> {
    let state = state.lock().await;

    let cpu = *state.cpu_usage.lock().unwrap();
    let mem = *state.mem_useag.lock().unwrap();
    let swap = *state.swap_usage.lock().unwrap();

    let disks: Vec<Value> = state
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

///<h1>---Get_Open_Port---</h1>
///<h4>read and found open and free port on your system</h4>
///
///<h3>by example</h3>
///```
///get_open_port(/* s:start_point, t:next_range */);
///
///```
///```
///get_open_port(3000,200);
///```
#[allow(dead_code)]
async fn get_open_port(s: u32, t: u32) -> u32 {
    for port in s..=(s + t) {
        if tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return port;
        }
    }
    panic!("no free port found range 0..65000");
}
