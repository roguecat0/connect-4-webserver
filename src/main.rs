use std::{collections::HashMap, ops::IndexMut};

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
            moves: "".to_string(),
        },
    })
}

async fn game(
    Path(moves): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mut game = GameHtml {
        values: [[""; 6]; 7],
        scores: [-4; 7],
        moves: format!("{}{}", &moves[1..], params.get("column").unwrap()),
    };
    let pick: usize = params.get("column").unwrap().parse().unwrap();

    println!("moves: {}", game.moves);
    for (ci, col) in game.values.iter_mut().enumerate() {
        for (i, ii) in game
            .moves
            .chars()
            .enumerate()
            .flat_map(|(ii, c)| {
                c.to_digit(10)
                    .and_then(|d| (d as usize == ci).then_some(ii))
            })
            .enumerate()
        {
            col[i] = if ii % 2 != 0 { "yellow" } else { "red" };
        }
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
    moves: String,
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
