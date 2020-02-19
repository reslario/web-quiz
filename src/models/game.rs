use {
    std::collections::VecDeque,
    diesel::{PgConnection, QueryResult},
    rand::{
        thread_rng,
        seq::SliceRandom
    },
    crate::{
        routing::play::Answer,
        models::stopwatch::Stopwatch,
        models::db::models::{Question, Category}
    }
};

#[derive(Debug)]
pub enum QuestionError {
    Query(diesel::result::Error),
    NoneRemaining
}

#[derive(Debug, Default)]
pub struct GameState {
    pub user: String,
    pub categories: Vec<Category>,
    pub current_question: Option<Question>,
    pub questions: VecDeque<Question>,
    pub points: i32,
    pub joker: bool,
    pub stopwatch: Stopwatch,
    answered: Vec<i32>
}

impl GameState {
    pub fn new(user: String, categories: Vec<Category>) -> GameState {
        GameState {
            user,
            categories,
            joker: true,
            stopwatch: Stopwatch::start(),
            ..<_>::default()
        }
    }

    pub fn next_question(&mut self) -> Option<(&Category, &Question)> {
        self.current_question
            .as_ref()
            .map(Question::id)
            .map(|id| self.answered.push(id));
        self.current_question = self.questions.pop_front();
        self.current_question()
    }

    pub fn current_question(&self) -> Option<(&Category, &Question)> {
        self.current_question
            .as_ref()
            .and_then(|q| self.categories
                .iter()
                .find(|cat| q.is_of_category(cat))
                .map(|cat| (cat, q))
            )
    }

    pub fn load_more_questions(&mut self, conn: &PgConnection) -> Result<(), QuestionError> {
        Question::load_set(&self.categories, &self.answered, conn)
            .map_err(QuestionError::Query)
            .into_iter()
            .filter(|vec| !vec.is_empty())
            .next()
            .ok_or(QuestionError::NoneRemaining)
            .map(|mut questions| self.questions
                .extend({
                    questions.shuffle(&mut thread_rng());
                    questions
                })
            )
    }

    pub fn increment_points(&mut self) {
        self.points += 30
    }

    pub fn use_joker(&mut self) -> Result<(), AlreadyUsed> {
        if !self.joker {
            Err(AlreadyUsed)
        } else {
            Ok(self.joker = false)
        }
    }

    pub fn weighted_points(&self) -> i32 {
        (self.points as u64 / self
            .stopwatch
            .elapsed()
            .as_secs())
        as i32
    }
}

pub struct AlreadyUsed;

pub fn pseudo_shuffle(items: &mut [Answer]) {
    items.sort_by_cached_key(|a| a.string
        .chars()
        .map(|c| c as usize)
        .sum::<usize>()
        / a.string.len().max(1)
    )
}

pub fn correct_ratio(question: &Question, conn: &PgConnection) -> QueryResult<u8> {
    question
        .stats()
        .load(conn)
        .map(|stats| stats.correct_ratio())
}