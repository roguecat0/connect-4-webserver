use askama::Template;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

#[derive(Clone, Copy)]
pub enum Cell {
    Empty,
    Red,
    Yellow,
}
impl Cell {
    pub fn into_html_class(&self) -> &'static str {
        match self {
            Cell::Empty => "",
            Cell::Yellow => "yellow",
            Cell::Red => "red",
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/game/:moves", get(game))
        .nest_service("/public", ServeDir::new("public"));
    let ip = "0.0.0.0:8088";

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();

    println!("server started on addr: {ip}");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    HtmlTemplate(Index {
        game: GameHtml {
            values: [[""; 6]; 7],
            scores: [-4; 7],
        },
    })
}

async fn game() -> impl IntoResponse {
    let mut game = GameHtml {
        values: [[""; 6]; 7],
        scores: [-4; 7],
    };
    game.values[0][0] = "yellow";
    game.values[1][0] = "red";
    for mut col in &mut game.values {
        col.reverse();
    }
    HtmlTemplate(game)
}
struct Game;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    game: GameHtml,
}

#[derive(Template)]
#[template(path = "game.html")]
struct GameHtml {
    values: [[&'static str; 6]; 7],
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
