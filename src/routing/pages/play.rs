use {
    serde::Serialize,
    chrono::Datelike,
    diesel::QueryResult,
    super::start::Settings,
    std::time::{Duration, SystemTime},
    rocket_contrib::{
        json::Json,
        templates::Template
    },
    rocket::{
        get,
        post,
        FromForm,
        response::Redirect,
        http::{Status, RawStr},
        request::{Form, FormItems, FromFormValue}
    },
    crate::models::{
        web::{NewSession, NewGameState, SyncedGameState, Or500, EndGame},
        game::{GameState, QuestionError, JokerError, pseudo_shuffle, correct_ratio},
        db::{
            DbConn,
            CategoryId,
            models::{Category, Question, Score}
        }
    }
};

#[derive(Serialize)]
struct DisplayData<'a> {
    question: &'a str,
    answers: Vec<&'a str>,
    category: &'a str,
    points: i32,
    joker: bool,
    ratio: u8,
    elapsed_secs: u64
}

impl <'a> DisplayData<'a> {
    pub fn new(question: &'a Question, category: &'a str, points: i32, joker: bool, ratio: u8, elapsed_secs: u64) -> DisplayData<'a> {
        let mut answers = question
            .incorrect
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(question.correct.as_str()))
            .collect::<Vec<_>>();
        pseudo_shuffle(&mut answers);

        DisplayData {
            question: &question.string,
            answers,
            category,
            points,
            joker,
            ratio,
            elapsed_secs
        }
    }
}

#[post("/play/new_game", data = "<settings>")]
pub fn new_game(settings: Form<Settings>, _sess: NewSession, new_game_state: NewGameState, conn: DbConn) -> Result<Redirect, Status> {
    let categories = Category::load_with_ids(&settings.categories, &conn)
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

#[post("/play/answer", data = "<response>")]
pub fn answer(response: Form<Response>, mut game_state: SyncedGameState, conn: DbConn) -> Result<Redirect, Status> {
    let cq = game_state
        .current_question
        .as_ref()
        .or_500()?;
    if response.answer == cq.correct {
        cq.stats()
            .add_correct(&conn)
            .or_500()?;
        drop(cq);
        game_state.increment_points();
        Ok(Redirect::to("/play"))
    } else {
        cq.stats()
            .add_incorrect(&conn)
            .or_500()?;
        Ok(Redirect::to("/play/end"))
    }
}

#[derive(Debug, Serialize)]
struct Intermission<'a> {
    categories: &'a [Category],
    points: i32,
    joker: bool
}

#[get("/play")]
pub fn continue_game(mut game_state: SyncedGameState, conn: DbConn) -> Result<Template, Status> {
    let (points, joker) = (game_state.points, game_state.joker);
    let elapsed_secs = game_state
        .stopwatch
        .elapsed()
        .as_secs();
    let (cat, next_q) = match game_state.next_question() {
        Some(v) => v,
        None => match game_state.load_more_questions(&conn) {
            Ok(()) => game_state
                .next_question()
                .or_500()?,
            Err(e) => return match e {
                QuestionError::Query(_) =>
                    Err(Status::InternalServerError),
                QuestionError::NoneRemaining =>
                    intermission(&mut game_state, &conn)
                        .or_500()
            }
        }
    };

    let ratio = correct_ratio(next_q, &conn)
        .or_500()?;

    Ok(Template::render("play", DisplayData::new(
        next_q,
        &cat.name,
        points,
        joker,
        ratio,
        elapsed_secs
    )))
}

fn intermission(game_state: &mut SyncedGameState, conn: &DbConn) -> QueryResult<Template> {
    game_state.stopwatch.pause();
    Category::load_all(conn)
        .map(|categories| render_intermission(&game_state, &categories))

}

fn render_intermission(game_state: &SyncedGameState, categories: &[Category]) -> Template {
     Template::render("play_error", Intermission {
        categories,
        points: game_state.points,
        joker: game_state.joker
     })
}

#[derive(Debug)]
pub struct NewCategories {
    categories: Vec<CategoryId>
}

impl <'f> rocket::request::FromForm<'f> for NewCategories {
    type Error = &'f RawStr;

    fn from_form(it: &mut FormItems<'f>, strict: bool) -> Result<Self, Self::Error> {
        it.map(|fi| fi.key_value())
            .filter_map(|(key, val)| match &*key.url_decode_lossy() {
                "categories" => CategoryId::from_form_value(val).into(),
                _ if strict => Err(val).into(),
                _ => None
            })
            .collect::<Result<_, _>>()
            .map(|categories| NewCategories {
                categories
            })
    }
}

#[post("/play/resume", data = "<new>")]
pub fn resume(new: Form<NewCategories>, mut game_state: SyncedGameState, conn: DbConn) -> Result<Redirect, Status> {
    game_state.stopwatch.resume();
    Category::load_with_ids(&new.categories, &conn)
        .map(|cats| game_state.set_categories(cats))
        .or_500()?;

    Ok(Redirect::to("/play"))
}

#[derive(Serialize)]
struct Results {
    user_score: DisplayScore,
    placement: u64,
    higher: Vec<DisplayScore>,
    lower: Vec<DisplayScore>,
    top_three: Vec<DisplayScore>
}

#[derive(Serialize)]
struct DisplayScore {
    name: String,
    points: i32,
    weighted_points: i32,
    played_on: Ymd,
    duration: Hms,
    categories: Vec<Category>
}

impl DisplayScore {
    pub fn from_score(score: Score, conn: &DbConn) -> QueryResult<DisplayScore> {
        let categories = Category::load_with_ids(&score.categories, conn)?;
        Ok(DisplayScore {
            name: score.name,
            points: score.points,
            weighted_points: score.weighted_points,
            played_on: score.played_on.into(),
            duration: score.duration.into(),
            categories
        })
    }
}

#[derive(Serialize)]
struct Ymd {
    y: u16,
    m: u8,
    d: u8
}

impl From<SystemTime> for Ymd {
    fn from(st: SystemTime) -> Self {
        let timestamp = st
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let date = chrono::naive::NaiveDateTime::from_timestamp(
            timestamp.as_secs() as _,
            0
        ).date();

        Ymd {
            y: date.year() as _,
            m: date.month() as _,
            d: date.day() as _
        }
    }
}

#[derive(Serialize)]
struct Hms {
    h: u32,
    m: u32,
    s: u32
}

impl From<Duration> for Hms {
    fn from(dur: Duration) -> Self {
        let dur = chrono::Duration::from_std(dur)
            .unwrap_or(chrono::Duration::zero());
        let h = dur.num_hours();
        let m = dur.num_minutes() - h * 60;
        let s = dur.num_seconds() - m * 60;

        Hms {
            h: h as _,
            m: m as _,
            s: s as _
        }
    }
}

#[get("/play/end")]
pub fn end_game(end: EndGame, conn: DbConn) -> Result<Template, Status> {
    let score = Score::insert(
        &end.game_state.score(),
        &conn
    ).or_500()?;

    let placement = score.placement(&conn).or_500()?;
    let (higher, lower) = score.neighbours(&conn).or_500()?;
    let top_three = Score::top_three(&conn).or_500()?;

    let user_score = DisplayScore::from_score(score, &conn).or_500()?;
    let higher = display_scores(higher, &conn).or_500()?;
    let lower = display_scores(lower, &conn).or_500()?;
    let top_three = display_scores(top_three, &conn).or_500()?;

    Ok(Template::render("end", Results {
        user_score,
        placement,
        higher,
        lower,
        top_three
    }))
}

fn display_scores(scores: Vec<Score>, conn: &DbConn) -> QueryResult<Vec<DisplayScore>> {
    scores
        .into_iter()
        .map(|score| DisplayScore::from_score(score, &conn))
        .collect()
}

#[derive(Serialize, Debug)]
pub struct Joker {
    incorrect: [String; 2]
}

#[get("/play/use_joker")]
pub fn use_joker(mut game_state: SyncedGameState) -> Result<Json<Joker>, Status> {
    game_state.use_joker()
        .map(|[ans1, ans2]| Joker {
            incorrect: [ans1.into(), ans2.into()]
        })
        .map(Json)
        .map_err(|e| match e {
            JokerError::AlreadyUsed => Status::Unauthorized,
            JokerError::NoQuestion => Status::InternalServerError
        })
}