use {
    diesel::{Identifiable, Queryable, Associations},
    super::schema::{questions, question_stats, categories, scores}
};

const NUM_ANSWERS: usize = 4;
const NUM_INCORRECT: usize = NUM_ANSWERS - 1;

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Category)]
#[table_name = "questions"]
pub struct Question {
    pub(super) id: i32,
    pub(super) category_id: i32,
    pub string: String,
    pub correct: String,
    #[diesel(deserialize_as = "Incorrect")]
    pub incorrect: [String; NUM_INCORRECT]
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Question)]
#[table_name = "question_stats"]
pub struct QuestionStats {
    pub(super) id: i32,
    pub(super) question_id: i32,
    pub num_correct: i32,
    pub num_incorrect: i32
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "categories"]
pub struct Category {
    pub(super) id: i32,
    pub name: String
}

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name = "scores"]
pub struct Score {
    pub(super) id: i32,
    pub name: String,
    pub points: i32
}
}
