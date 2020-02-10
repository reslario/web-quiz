use crate::models::db::models::{Question, Category};
use std::collections::VecDeque;
use diesel::{PgConnection, QueryResult};
use rand::{
    thread_rng,
    seq::SliceRandom
};
use crate::routing::Answer;

#[derive(Debug, Default)]
pub struct GameState {
    pub user: String,
    pub categories: Vec<Category>,
    pub current_question: Option<Question>,
    pub questions: VecDeque<Question>,
    pub points: i32,
    pub joker: bool
}

impl GameState {
    pub fn new(user: String, categories: Vec<Category>) -> GameState {
        GameState {
            user,
            categories,
            joker: true,
            ..Default::default()
        }
    }

    pub fn next_question(&mut self) -> Option<(&Category, &Question)> {
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

    pub fn load_more_questions(&mut self, conn: &PgConnection) -> QueryResult<()> {
        //TODO these need to be different than the ones before
        Question::load_set(&self.categories, conn)
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