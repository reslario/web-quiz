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

pub use {
    load::*,
    insert::*
};

mod load {
    use {
        super::NUM_INCORRECT,
        diesel::{
            Queryable,
            backend::Backend
        }
    };

    #[derive(PartialEq, Debug)]
    pub struct Incorrect([String; NUM_INCORRECT]);

    impl Into<[String; NUM_INCORRECT]> for Incorrect {
        fn into(self) -> [String; NUM_INCORRECT] {
            self.0
        }
    }

    impl <DB, ST> Queryable<ST, DB> for Incorrect
        where
            DB: Backend,
            Vec<String>: Queryable<ST, DB>,
    {
        type Row = <Vec<String> as Queryable<ST, DB>>::Row;

        fn build(row: Self::Row) -> Self {
            let mut answers = Vec::build(row)
                .into_iter();
            let mut next_answer = || answers
                .next()
                .unwrap_or_default();
            Incorrect([
                next_answer(),
                next_answer(),
                next_answer()
            ])
        }
    }

}

mod insert {
    use {
        super::*,
        diesel::Insertable
    };

    #[derive(Insertable, Debug, PartialEq, PartialOrd, Clone)]
    #[table_name = "questions"]
    pub struct NewQuestion<'a> {
        pub category_id: i32,
        pub string: &'a str,
        pub correct: &'a str,
        pub incorrect: &'a [String]
    }

    #[derive(Insertable, Debug, PartialEq, PartialOrd, Clone)]
    #[table_name = "question_stats"]
    pub struct NewQuestionStats {
        pub question_id: i32,
    }

    #[derive(Insertable, Debug, PartialEq, PartialOrd, Clone)]
    #[table_name = "categories"]
    pub struct NewCategory<'a> {
        pub name: &'a str
    }

    #[derive(Insertable, Debug, PartialEq, PartialOrd, Clone)]
    #[table_name = "scores"]
    pub struct NewScore<'a> {
        pub name: &'a str,
        pub points: i32
    }
}
