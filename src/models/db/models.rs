use {
    serde::{Serialize, Deserialize},
    std::time::{Duration, SystemTime},
    diesel::{Identifiable, Queryable, Associations},
    super::{
        CategoryId,
        schema::{questions, question_stats, categories, scores, admins}
    }
};

const NUM_ANSWERS: usize = 4;
const NUM_INCORRECT: usize = NUM_ANSWERS - 1;

#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug, Clone)]
#[belongs_to(Question)]
#[table_name = "question_stats"]
pub struct QuestionStats {
    pub(super) id: i32,
    pub(super) question_id: i32,
    pub num_correct: i32,
    pub num_incorrect: i32
}

#[derive(Identifiable, Queryable, Serialize, PartialEq, Eq, Hash, Debug, Clone)]
#[table_name = "categories"]
pub struct Category {
    pub(super) id: i32,
    pub name: String
}

#[derive(Identifiable, Queryable, Serialize, PartialEq, Debug, Clone)]
#[table_name = "scores"]
pub struct Score {
    pub(super) id: i32,
    pub name: String,
    pub points: i32,
    pub weighted_points: i32,
    pub played_on: SystemTime,
    #[diesel(deserialize_as = "DurationSecs")]
    pub duration: Duration,
    #[diesel(deserialize_as = "ScoreCategories")]
    pub categories: Vec<CategoryId>
}

#[derive(Identifiable, Queryable, PartialEq, Debug, Clone)]
#[table_name = "admins"]
pub struct Admin {
    pub(super) id: i32,
    pub name: String,
    pub password: String
}

pub use {
    load::*,
    insert::*
};

mod load {
    use {
        super::*,
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

    #[derive(PartialEq, Debug)]
    pub struct DurationSecs(Duration);

    impl Into<Duration> for DurationSecs {
        fn into(self) -> Duration {
            self.0
        }
    }

    impl <DB, ST> Queryable<ST, DB> for DurationSecs
        where
            DB: Backend,
            i64: Queryable<ST, DB>,
    {
        type Row = <i64 as Queryable<ST, DB>>::Row;

        fn build(row: Self::Row) -> Self {
            DurationSecs(
                Duration::from_secs(
                    i64::build(row) as u64
                )
            )
        }
    }

    pub struct ScoreCategories(Vec<CategoryId>);

    impl Into<Vec<CategoryId>> for ScoreCategories {
        fn into(self) -> Vec<CategoryId> {
            self.0
        }
    }

    impl <DB, ST> Queryable<ST, DB> for ScoreCategories
        where
            DB: Backend,
            Vec<i32>: Queryable<ST, DB>,
    {
        type Row = <Vec<i32> as Queryable<ST, DB>>::Row;

        fn build(row: Self::Row) -> Self {
            ScoreCategories(
                Vec::build(row)
                    .into_iter()
                    .map(CategoryId)
                    .collect()
            )
        }
    }
}

mod insert {
    use {
        super::*,
        diesel::Insertable
    };

    #[derive(Insertable, AsChangeset, Debug, PartialEq, PartialOrd, Clone)]
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
        pub points: i32,
        pub weighted_points: i32,
        pub played_on: SystemTime,
        #[column_name = "duration"]
        pub duration_secs: i64,
        pub categories: Vec<CategoryId>
    }

    #[derive(Insertable, Debug, PartialEq, PartialOrd, Clone)]
    #[table_name = "admins"]
    pub struct NewAdmin<'a> {
        pub name: &'a str,
        pub password: &'a str
    }
}
