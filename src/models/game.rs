use crate::models::db::models::{Question, Category};
use std::collections::VecDeque;
use diesel::{PgConnection, QueryResult};
use rand::{
    thread_rng,
    seq::SliceRandom
};
use std::iter::Sum;
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
        let categories = &self.categories;
        self.current_question
            .as_ref()
            .and_then(|q| categories
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

    pub fn use_joker(&mut self) {
        self.joker = false
    }
}

pub fn pseudo_shuffle(items: &mut [Answer])
{
    items.sort_by_cached_key(|a| a.string
        .chars()
        .map(|c| c as usize)
        .sum::<usize>()
        / a.string.len().min(1)
    )
}