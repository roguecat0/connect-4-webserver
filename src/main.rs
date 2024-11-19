use connect_4_ai::{OpeningBook, Position, Solver};
use std::{collections::HashMap, sync::Arc, usize};

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
        .route("/game/:moves", get(game))
        .route("/toggle_show", get(toggle_show))
        .route("/yellow", get(start_yellow))
        .route("/red", get(start_red))
        .nest_service("/public", ServeDir::new("public"));
    let ip = "0.0.0.0:8088";

    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();

    println!("server started on addr: {ip}");
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> impl IntoResponse {
    HtmlTemplate(Index {
        game: GameHtml::new(),
    })
}
#[derive(Debug)]
enum GameStatus {
    Winning(usize),
    Losing(usize),
    Drawing(usize),
}

impl GameStatus {
    pub fn new(score: isize, num_moves: usize) -> Self {
        let state = score.checked_div(score.abs()).unwrap_or(0);
        println!("div: {:?}, score: {score}", score.checked_div(score.abs()));
        let moves_left = 21 - num_moves / 2 - score.abs() as usize;
        println!("state: {state}, num_moves: {num_moves}, m_left: {moves_left}");
        match (moves_left, state) {
            (n, 0) => GameStatus::Drawing(n),
            (n, 1) => GameStatus::Winning(n),
            (n, -1) => GameStatus::Losing(n),
            n => panic!("not a possible senario: {n:?}"),
        }
    }
    pub fn reverse(&self) -> Self {
        match self {
            GameStatus::Losing(n) => GameStatus::Winning(*n),
            GameStatus::Drawing(n) => GameStatus::Drawing(*n),
            GameStatus::Winning(n) => GameStatus::Losing(*n),
        }
    }
    pub fn to_msg(&self) -> String {
        match self {
            GameStatus::Losing(0) => format!("You Lost"),
            GameStatus::Losing(n) => format!("You will lost in {n}, turn(s)"),
            GameStatus::Drawing(0) => format!("You Drew"),
            GameStatus::Drawing(n) => format!("You can draw in {n}, turn(s)"),
            GameStatus::Winning(0) => format!("You Won"),
            GameStatus::Winning(n) => format!("You can win in {n}, turn(s)"),
        }
    }
    pub fn is_reset(&self) -> bool {
        match self {
            GameStatus::Losing(0) | GameStatus::Drawing(0) | GameStatus::Winning(0) => true,
            _ => false,
        }
    }
}

async fn toggle_show(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    println!("{:?}", params);
    let moves = params.get("moves").unwrap();
    let show_scores: bool = params
        .get("show_scores")
        .expect("show_scores to be there")
        .parse()
        .expect("string to have a boolean value");
    let book = Arc::new(OpeningBook::load("7x6.book").unwrap());
    let mut solver = Solver::with_opening_book(book);

    // coninute evaluating
    let position = Position::parse_safe(moves).unwrap();
    let scores = solver.analyse(&position, false);
    println!("scores you: {scores:?}");
    let game = if let Some(best_move) = get_best_move(&scores) {
        let status = GameStatus::new(best_move.1, moves.len().checked_sub(2).unwrap_or(0));
        println!("you status: {:?}", status);
        GameHtml::from(
            status,
            &moves,
            transform_scores_array(&scores),
            !show_scores,
        )
    } else {
        GameHtml::from(
            GameStatus::Drawing(0),
            &moves,
            default_string_array(),
            !show_scores,
        )
    };
    println!("");
    HtmlTemplate(game)
}
async fn start_yellow() -> impl IntoResponse {
    HtmlTemplate(GameHtml::new_second())
}
async fn start_red() -> impl IntoResponse {
    HtmlTemplate(GameHtml::new())
}

async fn game(
    Path(moves): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // check if game end (win, draw)
    let moves = &moves[1..];
    println!("{:?}", params);
    let pick: usize = params.get("column").unwrap().parse().unwrap();
    let show_scores: bool = params
        .get("show_scores")
        .expect("show_scores to be there")
        .parse()
        .expect("string to have a boolean value");
    let pos = Position::parse_safe(&moves).unwrap();
    if pos.is_winning_move(pick) {
        println!("you won!! {pick}");
        return HtmlTemplate(GameHtml::from(
            GameStatus::Winning(0),
            moves,
            default_string_array(),
            show_scores,
        ));
    } else if moves.len() == 41 {
        println!("you drew!! {pick}");
        return HtmlTemplate(GameHtml::from(
            GameStatus::Drawing(0),
            moves,
            default_string_array(),
            show_scores,
        ));
    }

    // check if (lost, draw)
    let moves = format!("{moves}{pick}");
    let book = Arc::new(OpeningBook::load("7x6.book").unwrap());
    let mut solver = Solver::with_opening_book(book);

    let position = pos.next_pos_safe(pick);
    if let None = position {
        println!("failed to parse moves: {}", moves);
        return HtmlTemplate(GameHtml::new());
    }
    let position = position.unwrap();
    let scores = solver.analyse(&position, false);
    println!("scores bot: {scores:?}");
    let best_move = if let Some(best_move) = get_best_move(&scores) {
        println!(
            "you status: {:?}",
            GameStatus::new(best_move.1, moves.len())
        );
        if let GameStatus::Winning(0) | GameStatus::Drawing(0) =
            GameStatus::new(best_move.1, moves.len())
        {
            println!(
                "you lost or drew!! {} {:?}",
                moves,
                GameStatus::new(best_move.1, moves.len())
            );
            return HtmlTemplate(GameHtml::from(
                GameStatus::new(best_move.1, moves.len()).reverse(),
                &format!("{moves}{}", best_move.0),
                default_string_array(),
                show_scores,
            ));
        }
        best_move
    } else {
        println!("bot drew!! {pick}");
        return HtmlTemplate(GameHtml::from(
            GameStatus::Drawing(0),
            &moves,
            default_string_array(),
            show_scores,
        ));
    };
    // coninute evaluating
    let position = position.next_pos(best_move.0);
    let moves = format!("{}{}", moves, best_move.0);
    let scores = solver.analyse(&position, false);
    println!("scores you: {scores:?}");
    let game = if let Some(best_move) = get_best_move(&scores) {
        let status = GameStatus::new(best_move.1, moves.len() - 2);
        println!("you status: {:?}", status);
        GameHtml::from(status, &moves, transform_scores_array(&scores), show_scores)
    } else {
        println!("bot drew!! {pick}");
        GameHtml::from(
            GameStatus::Drawing(0),
            &moves,
            default_string_array(),
            show_scores,
        )
    };
    println!("");
    HtmlTemplate(game)
}
fn get_board_css(moves: &str) -> [[&'static str; 6]; 7] {
    let mut values = [[""; 6]; 7];
    for (ci, col) in values.iter_mut().enumerate() {
        for (i, ii) in moves
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
    values
}
fn get_best_move(scores: &[Option<isize>]) -> Option<(usize, isize)> {
    scores
        .into_iter()
        .enumerate()
        .flat_map(|(i, c)| c.and_then(|cc| Some((i, cc))))
        .reduce(|old, new| if old.1 > new.1 { old } else { new })
}

fn default_string_array() -> [String; 7] {
    [
        "None".to_string(),
        "None".to_string(),
        "None".to_string(),
        "None".to_string(),
        "None".to_string(),
        "None".to_string(),
        "None".to_string(),
    ]
}

fn transform_scores_array(scores: &[Option<isize>]) -> [String; 7] {
    fn trans(score: &Option<isize>) -> String {
        score.map_or("None".to_string(), |d| d.to_string())
    }
    let mut output = default_string_array();
    for (i, score) in scores.into_iter().enumerate() {
        output[i] = trans(score);
    }
    output
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    game: GameHtml,
}

#[derive(Template)]
#[template(path = "game.html")]
struct GameHtml {
    values: [[&'static str; 6]; 7],
    scores: [String; 7],
    path: String,
    status_msg: String,
    show_scores: bool,
}
impl GameHtml {
    pub fn new() -> Self {
        Self {
            values: [[""; 6]; 7],
            scores: default_string_array(),
            path: "/game/0".to_string(),
            status_msg: "Game Started".to_string(),
            show_scores: false,
        }
    }
    pub fn new_second() -> Self {
        let mut values = [[""; 6]; 7];
        values[3][5] = "red";
        Self {
            values,
            scores: default_string_array(),
            path: "/game/03".to_string(),
            status_msg: "Game Started".to_string(),
            show_scores: false,
        }
    }

    pub fn from(status: GameStatus, moves: &str, scores: [String; 7], show_scores: bool) -> Self {
        Self {
            values: get_board_css(moves),
            scores,
            path: Self::generate_path(moves, &status),
            status_msg: status.to_msg(),
            show_scores,
        }
    }
    fn generate_path(moves: &str, status: &GameStatus) -> String {
        if status.is_reset() {
            "/game/0".to_string()
        } else {
            format!("/game/0{}", moves)
        }
    }
    fn is_reset(&self) -> bool {
        &self.path == "/game/0"
    }
    fn get_moves(&self) -> String {
        (&self.path[7..]).to_string()
    }
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
