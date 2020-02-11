use {
    super::Settings,
    serde::Serialize,
    rocket_contrib::templates::Template,
    rand::{
        thread_rng,
        seq::index::sample
    },
    rocket::{
        get,
        post,
        FromForm,
        http::Status,
        request::Form,
        response::Redirect,
    },
    crate::models::{
        game::{GameState, AlreadyUsed, pseudo_shuffle},
        web::{NewSession, NewGameState, SyncedGameState, Or500, EndGame},
        db::{
            DbConn,
            models::{Category, Question, Score, NewScore}
        }
    }
};
use crate::models::game::correct_ratio;

#[derive(Serialize)]
struct DisplayData<'a> {
    question: &'a str,
    answers: Vec<Answer<'a>>,
    category: &'a str,
    points: i32,
    joker: bool,
    ratio: u8
}

#[derive(Serialize)]
pub struct Answer<'a> {
    pub string: &'a str,
    disabled: bool
}

impl <'a> DisplayData<'a> {
    pub fn new(question: &'a Question, category: &'a str, points: i32, joker: bool, ratio: u8) -> DisplayData<'a> {
        let mut answers = question
            .incorrect
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(question.correct.as_str()))
            .map(|string| Answer { string, disabled: false })
            .collect::<Vec<_>>();
        pseudo_shuffle(&mut answers);

        DisplayData {
            question: &question.string,
            answers,
            category,
            points,
            joker,
            ratio
        }
    }

    pub fn apply_joker(&mut self, correct: &str) {
        let indices = sample(&mut thread_rng(), 3, 2);
            self.answers
                .iter_mut()
                .filter(|a| a.string != correct)
                .enumerate()
                .filter(|(i, _)| indices
                    .iter()
                    .find(|id| id == i)
                    .is_some()
                ).for_each(|(_, a)| a.disabled = true)
    }
}

#[post("/new_game", data = "<settings>")]
pub fn new_game(settings: Form<Settings>, _sess: NewSession, new_game_state: NewGameState, conn: DbConn) -> Result<Redirect, Status> {
    let categories = Category::load_with_ids(&settings.categories, &*conn)
        .or_500()?;

    new_game_state.set(GameState::new(
        settings.0.user,
        categories,
    ));

    Ok(Redirect::to("/play"))
}

#[derive(FromForm, Debug)]
pub struct Response {
    answer: String
}

#[post("/answer", data = "<response>")]
pub fn answer(response: Form<Response>, mut game_state: SyncedGameState, conn: DbConn) -> Result<Redirect, Status> {
    let cq = game_state
        .current_question
        .as_ref()
        .or_500()?;
    if response.answer == cq.correct {
        cq.stats()
            .add_correct(&*conn)
            .or_500()?;
        drop(cq);
        game_state.increment_points();
        Ok(Redirect::to("/play"))
    } else {
        cq.stats()
            .add_incorrect(&*conn)
            .or_500()?;
        Ok(Redirect::to("/end"))
    }
}

#[get("/play")]
pub fn continue_game(mut game_state: SyncedGameState, conn: DbConn) -> Result<Template, Status> {
    let (points, joker) = (game_state.points, game_state.joker);
    let (cat, next_q) = match game_state.next_question() {
        Some(v) => v,
        None => {
            game_state.load_more_questions(&*conn)
                .or_500()?;
            game_state.next_question()
                .or_500()?
        }
    };

    let ratio = correct_ratio(next_q, &*conn)
        .or_500()?;

    Ok(Template::render("play", DisplayData::new(
        next_q,
        &cat.name,
        points,
        joker,
        ratio
    )))
}

#[derive(Serialize)]
struct Results<'a> {
    user_score: &'a Score,
    placement: u64,
    higher: &'a [Score],
    lower: &'a [Score],
    top_three: &'a [Score]
}

#[get("/end")]
pub fn end_game(end: EndGame, conn: DbConn) -> Result<Template, Status> {
    let score = Score::insert(
        &NewScore {
            name: &end.game_state.user,
            points: end.game_state.points
        },
        &*conn
    ).or_500()?;

    let placement = score.placement(&*conn).or_500()?;
    let (higher, lower) = score.neighbours(&*conn).or_500()?;
    let top_three = Score::top_three(&*conn).or_500()?;

    Ok(Template::render("end", Results {
        user_score: &score,
        placement,
        higher: &higher,
        lower: &lower,
        top_three: &top_three
    }))
}

#[get("/use_joker")]
pub fn use_joker(mut game_state: SyncedGameState, conn: DbConn) -> Result<Template, Status> {
    let allowed = match game_state.use_joker() {
        Ok(()) => true,
        Err(AlreadyUsed) => false
    };
    let (cat, q) = game_state.current_question().or_500()?;

    let ratio = correct_ratio(q, &*conn)
        .or_500()?;

    let mut display_data = DisplayData::new(
        q,
        &cat.name,
        game_state.points,
        false,
        ratio
    );

    if allowed {
        display_data.apply_joker(&q.correct)
    }

    Ok(Template::render("play", display_data))
}