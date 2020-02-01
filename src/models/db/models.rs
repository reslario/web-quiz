use {
    diesel::{Identifiable, Queryable, Associations},
    super::schema::{questions, question_stats, categories, scores}
};

const NUM_ANSWERS: usize = 4;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Category)]
#[table_name = "questions"]
pub struct Question {
    id: u32,
    category_id: u32,
    string: String,
    correct: String,
    incorrect: [String; NUM_ANSWERS - 1]
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Question)]
#[table_name = "question_stats"]
pub struct QuestionStats {
    id: u32,
    question_id: u32,
    num_correct: u32,
    num_incorrect: u32
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "categories"]
pub struct Category {
    id: u32,
    name: String
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "scores"]
pub struct Score {
    id: u32,
    user: String,
    points: u32
}
