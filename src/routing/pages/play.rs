use {
    super::Settings,
    serde::Serialize,
    rocket_contrib::templates::Template,
    rand::{
        thread_rng,
        seq::{
            SliceRandom,
            index::sample
        }
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
        game::{GameState, pseudo_shuffle},
        web::{NewSession, NewGameState, MutSyncedGameState, Or500, EndSession, SyncedGameState},
        db::{
            DbConn,
            models::{Category, Question, Score, NewScore}
        }
    }
};

#[derive(Serialize)]
struct DisplayData<'a> {
    question: &'a str,
    answers: Vec<Answer<'a>>,
    category: &'a str,
    points: i32,
    joker: bool
}

#[derive(Serialize)]
pub struct Answer<'a> {
    pub string: &'a str,
    disabled: bool
}

impl <'a> DisplayData<'a> {
    pub fn new(question: &'a Question, category: &'a str, points: i32, joker: bool) -> DisplayData<'a> {
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
            joker
        }
    }

    pub fn apply_joker(&mut self, correct: &str) {
        let indices = sample(&mut thread_rng(), 3, 2);
            self.answers
                .iter_mut()
                .filter(|a| a.string != correct)
                .enumerate()
                .filter(|(i, a)| indices
                    .iter()
                    .find(|id| id == i)
                    .is_some()
                ).for_each(|(i, a)| a.disabled = true)
    }
}

#[post("/new_game", data = "<settings>")]
pub fn new_game(settings: Form<Settings>, _sess: NewSession, new_game_state: NewGameState, conn: DbConn) -> Result<Template, Status> {
    let categories = Category::load_with_ids(&settings.categories, &*conn)
        .or_500()?;

    let mut game_state = GameState::new(
        settings.0.user,
        categories,
    );

    game_state.load_more_questions(&*conn)
        .or_500()?;
    let (points, joker) = (game_state.points, game_state.joker);
    let (cat, next_q) = game_state
        .next_question()
        .or_500()?;

    let response = Template::render("play", DisplayData::new(
        next_q,
        &cat.name,
        points,
        joker
    ));

    new_game_state.set(game_state);

    Ok(response)
}

#[derive(FromForm, Debug)]
pub struct Response {
    answer: String
}

#[post("/answer", data = "<answer>")]
pub fn answer(answer: Form<Response>, game_state: MutSyncedGameState) -> Redirect {
    if game_state.current_question
        .as_ref()
        .map(|q| q.correct == answer.answer)
        .unwrap_or_default()
    {
        Redirect::to("/play")
    } else {
        Redirect::to("/end")
    }
}

#[get("/play")]
pub fn continue_game(mut game_state: MutSyncedGameState, conn: DbConn) -> Result<Template, Status> {
    game_state.increment_points();
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

    Ok(Template::render("play", DisplayData::new(
        next_q,
        &cat.name,
        points,
        joker
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
pub fn end_game(game_state: SyncedGameState, conn: DbConn, _sess: EndSession) -> Result<Template, Status> {
    let score = Score::insert(
        &NewScore {
            name: &game_state.user,
            points: game_state.points
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