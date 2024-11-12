use askama::Template;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .nest_service("/public", ServeDir::new("public"));
    let ip = "0.0.0.0:8088";

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();

    println!("server started on addr: {ip}");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    HtmlTemplate(Index {
        game: Game {
            values: [[0; 6]; 7],
            scores: [-4; 7],
        },
    })
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    game: Game,
}

#[derive(Template)]
#[template(path = "game.html")]
struct Game {
    values: [[u8; 6]; 7],
    scores: [isize; 7],
}

// render templates
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => (StatusCode::OK, Html(html)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("template err: {e}"),
            )
                .into_response(),
        }
    }
}
