use axum::{extract::Path, response::{IntoResponse, Json}, routing::get, Router};
use common::VideoList;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedVideos = Arc<RwLock<VideoList>>;

/// Charge la liste des vidéos depuis le JSON
fn load_videos_from_file(path: &str) -> VideoList {
    let data = fs::read_to_string(path).expect("Impossible de lire videos.json");
    serde_json::from_str(&data).expect("JSON invalide")
}

async fn list_videos(videos: axum::extract::State<SharedVideos>) -> Json<VideoList> {
    let v = videos.read().await;
    Json(v.clone())
}

async fn get_video(
    Path(id): Path<String>,
    videos: axum::extract::State<SharedVideos>
) -> impl IntoResponse {
    let v = videos.read().await;
    if let Some(video) = v.videos.iter().find(|vid| vid.id == id) {
        Json(serde_json::json!({
            "id": video.id,
            "title": video.title,
            "url": video.url
        }))
    } else {
        Json(serde_json::json!({
            "error": "Video not found"
        }))
    }
}

#[tokio::main]
async fn main() {
    // Charger les vidéos au démarrage
    let videos = load_videos_from_file("backend/config/videos.json");
    let shared_videos = Arc::new(RwLock::new(videos));

    let app = Router::new()
        .route("/videos", get(list_videos))
        .route("/videos/{id}", get(get_video))
        .with_state(shared_videos);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Backend running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .unwrap();
}
