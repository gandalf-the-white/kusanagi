use axum::{extract::Path, response::Html, routing::get, Router};
use common::VideoList;
use reqwest;
use serde_json::Value;

async fn list_videos() -> Html<String> {
    let proxy_url = "http://127.0.0.1:7070";

    let res = reqwest::get(format!("{}/videos", proxy_url))
        .await
        .unwrap()
        .json::<VideoList>()
        .await
        .unwrap();

    let mut html = String::from(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Liste des vidéos</title>
            <style>
                body { font-family: Arial, sans-serif; padding: 20px; }
                .video-list { list-style: none; padding: 0; }
                .video-list li { margin: 10px 0; }
                .play-btn {
                    background: #007bff; color: white; padding: 5px 10px;
                    border: none; border-radius: 4px; cursor: pointer;
                }
            </style>
        </head>
        <body>
            <h1>Liste des vidéos</h1>
            <ul class="video-list">
    "#);

    for v in res.videos {
        html.push_str(&format!(
            r#"<li>{} <a href="/play/{}"><button class="play-btn">Lire</button></a></li>"#,
            v.title, v.id
        ));
    }

    html.push_str("</ul></body></html>");
    Html(html)
}

async fn play_video(Path(id): Path<String>) -> Html<String> {
    let proxy_url = "http://127.0.0.1:7070";
    let res: Value = reqwest::get(format!("{}/videos/{}", proxy_url, id))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let video_src = res["url"].as_str().unwrap();

    let html = format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Lecture vidéo</title>
            <script src="https://cdn.jsdelivr.net/npm/hls.js@latest"></script>
            <style>
                body {{ font-family: Arial, sans-serif; text-align: center; padding: 20px; }}
                video {{ width: 80%; max-width: 800px; margin-top: 20px; }}
            </style>
        </head>
        <body>
            <h1>Lecture vidéo {id}</h1>
            <video id="video" controls></video>
            <script>
                if (Hls.isSupported()) {{
                    var video = document.getElementById('video');
                    var hls = new Hls();
                    hls.loadSource('{video_src}');
                    hls.attachMedia(video);
                }} else if (video.canPlayType('application/vnd.apple.mpegurl')) {{
                    video.src = '{video_src}';
                }}
            </script>
        </body>
        </html>
    "#);

    Html(html)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/videos", get(list_videos))
        .route("/play/{id}", get(play_video));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9090")
        .await
        .unwrap();

    println!("Frontend running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .unwrap();

}
