use {
    diesel::{
        insert_into,
        PgConnection,
        RunQueryDsl,
        prelude::*,
        result::QueryResult,
        expression::dsl::any
    },
    crate::models::db::{
        schema,
        models::{
            Question, NewQuestion,
            QuestionStats, NewQuestionStats,
            Category, NewCategory,
            Score, NewScore
        }
    }
};

impl Category {
    pub fn load_all(conn: &PgConnection) -> QueryResult<Vec<Category>> {
        use schema::categories::dsl::*;

        categories.load(&*conn)
    }

    pub fn load_with_ids(ids: &[i32], conn: &PgConnection) -> QueryResult<Vec<Category>> {
        use schema::categories::dsl::*;

        categories
            .filter(id.eq(any(ids)))
            .load(conn)
    }
}

impl Question {
    pub fn insert(new: &NewQuestion, conn: &PgConnection) -> QueryResult<Question> {
        use schema::questions::dsl::*;

        let res: Question = insert_into(questions)
            .values(new)
            .get_result(&*conn)?;

        QuestionStats::insert(
            &NewQuestionStats { question_id: res.id },
            conn
        )?;

        Ok(res)
    }

    pub fn is_of_category(&self, cat: &Category) -> bool {
        self.category_id == cat.id
    }

    pub(in crate::models) fn load_set(categories: &[Category], conn: &PgConnection) -> QueryResult<Vec<Question>> {
        Question::belonging_to(categories)
            .limit(100)
            .load(conn)
    }
}

impl Score {
    pub fn top_three(conn: &PgConnection) -> QueryResult<Vec<Score>> {
        use schema::scores::dsl::*;

        scores
            .order(points.desc())
            .limit(3)
            .load(conn)
    }

    pub fn placement(&self, conn: &PgConnection) -> QueryResult<u64> {
        use schema::scores::dsl::*;

        scores
            .filter(points.gt(self.points))
            .count()
            .get_result::<i64>(conn)
            .map(|count| count as u64 + 1)
    }

    pub fn neighbours(&self, conn: &PgConnection) -> QueryResult<(Vec<Score>, Vec<Score>)> {
        use schema::scores::dsl::*;

        let higher = scores
            .order(points.desc())
            .filter(points.gt(self.points))
            .limit(10)
            .load(conn)?;

        let lower = scores
            .order(points.desc())
            .filter(points.lt(self.points))
            .limit(10)
            .load(conn)?;

        Ok((higher, lower))
    }
}

macro_rules! impl_insert {
    ($vis:vis, $name:ident, $new:ident, $table:ident) => {
        impl $name {
            $vis fn insert(new: &$new, conn: &PgConnection) -> QueryResult<$name> {
                use schema::$table::dsl::*;

                insert_into($table)
                    .values(new)
                    .get_result(&*conn)
            }
        }
    };
    ($name:ident, $new:ident, $table:ident) => {
        impl_insert!( , $name, $new, $table);
    }
}

impl_insert!(QuestionStats, NewQuestionStats, question_stats);

impl_insert!(pub, Category, NewCategory, categories);

impl_insert!(pub, Score, NewScore, scores);