use {
    derive_more::Deref,
    serde::{Serialize, Deserialize},
    std::{
        io::Write,
        ops::{Add, Div, Mul}
    },
    crate::models::db::{
        schema,
        models::*
    },
    rocket::{
        http::RawStr,
        request::{
            FromParam,
            FromFormValue
        }
    },
    diesel::{
        update,
        delete,
        insert_into,
        PgConnection,
        AsExpression,
        RunQueryDsl,
        prelude::*,
        pg::Pg,
        backend::Backend,
        sql_types::Integer,
        result::QueryResult,
        expression::dsl::{any, all},
        serialize::{ToSql, Output},
        query_builder::{AsChangeset, QueryFragment}
    }
};

macro_rules! impl_to_sql_for_id {
    ($item:ident) => {
        impl<DB> ToSql<Integer, DB> for $item
        where
            DB: Backend,
            i32: ToSql<Integer, DB>
        {
            fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> diesel::serialize::Result {
                self.0.to_sql(out)
            }
        }
    }
}

macro_rules! impl_from_form_value_for_id {
    ($item:ident) => {
        impl<'v> FromFormValue<'v> for $item {
            type Error = &'v RawStr;

            fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
                i32::from_form_value(form_value)
                    .map($item)
            }
        }
    };
}

#[derive(AsExpression, Serialize, Deserialize, Deref, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[sql_type = "diesel::sql_types::Integer"]
pub struct CategoryId(pub(super) i32);

impl_to_sql_for_id!(CategoryId);
impl_from_form_value_for_id!(CategoryId);

impl Category {
    pub fn id(&self) -> CategoryId {
        CategoryId(self.id)
    }

    pub fn load_all(conn: &PgConnection) -> QueryResult<Vec<Category>> {
        use schema::categories::dsl::*;

        categories.load(conn)
    }

    pub fn load_with_ids(ids: &[CategoryId], conn: &PgConnection) -> QueryResult<Vec<Category>> {
        use schema::categories::dsl::*;

        categories
            .filter(id.eq(any(ids)))
            .load(conn)
    }
}

#[derive(AsExpression, Serialize, Deserialize, Deref, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[sql_type = "diesel::sql_types::Integer"]
pub struct QuestionId(pub(super) i32);

impl_to_sql_for_id!(QuestionId);
impl_from_form_value_for_id!(QuestionId);

impl <'r> FromParam<'r> for QuestionId {
    type Error = <i32 as FromParam<'r>>::Error;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        i32::from_param(param).map(QuestionId)
    }
}

impl Question {
    const PER_SET: usize = 100;

    pub fn id(&self) -> QuestionId {
        QuestionId(self.id)
    }

    pub fn category_id(&self) -> CategoryId {
        CategoryId(self.category_id)
    }

    pub fn insert(new: &NewQuestion, conn: &PgConnection) -> QueryResult<Question> {
        use schema::questions::dsl::*;

        let res: Question = insert_into(questions)
            .values(new)
            .get_result(conn)?;

        QuestionStats::insert(
            &NewQuestionStats { question_id: res.id },
            conn
        )?;

        Ok(res)
    }

    pub fn is_of_category(&self, cat: &Category) -> bool {
        self.category_id == cat.id
    }

    pub(in crate::models) fn load_set(categories: &[Category], answered: &[QuestionId], conn: &PgConnection) -> QueryResult<Vec<Question>> {
        use schema::questions::dsl::*;

        let per_category = (Self::PER_SET / categories.len()) as i64;
        let mut result = Vec::with_capacity(Self::PER_SET);

        // this might not always give us the full PER_SET, since
        // a category might not have enough unanswered questions
        // remaining, but it's whatever
        categories
            .iter()
            .map(|cat| Question::belonging_to(cat)
                .filter(id.ne(all(answered)))
                .limit(per_category)
                .load(conn)
                .map(|qs| result.extend(qs))
            ).collect::<Result<_, _>>()?;

        Ok(result)
    }

    pub fn load_all(conn: &PgConnection) -> QueryResult<Vec<Question>> {
        use schema::questions::dsl::*;

        questions.load(conn)
    }

    pub fn delete(qid: QuestionId, conn: &PgConnection) -> QueryResult<()> {
        use schema::questions::dsl::*;

        {
            use schema::question_stats::dsl::*;

            delete(question_stats.filter(question_id.eq(qid)))
                .execute(conn)?;
        }

        delete(questions.filter(id.eq(qid)))
            .execute(conn)
            .map(drop)
    }

    pub fn update(qid: QuestionId, new: NewQuestion, conn: &PgConnection) -> QueryResult<Question> {
        use schema::questions::dsl::*;

        update(questions.filter(id.eq(qid)))
            .set(new)
            .get_result(conn)
    }

    pub fn stats(&self) -> Stats {
        Stats {
            question: self
        }
    }
}

pub struct Stats<'a> {
    question: &'a Question
}

impl <'a> Stats<'a> {
    pub fn load(&self, conn: &PgConnection) -> QueryResult<QuestionStats> {
        QuestionStats::belonging_to(self.question)
            .get_result(conn)
    }

    pub fn add_correct(&self, conn: &PgConnection) -> QueryResult<()> {
        use schema::question_stats::dsl::*;

        self.update_stat(
            num_correct.eq(num_correct + 1),
            conn
        )
    }

    pub fn add_incorrect(&self, conn: &PgConnection) -> QueryResult<()> {
        use schema::question_stats::dsl::*;

        self.update_stat(
            num_incorrect.eq(num_incorrect + 1),
            conn
        )
    }

    fn update_stat<V>(&self, expr: V, conn: &PgConnection) -> QueryResult<()>
    where
        V: AsChangeset<Target = schema::question_stats::table>,
        <V as AsChangeset>::Changeset: QueryFragment<Pg>
    {
        update(QuestionStats::belonging_to(self.question))
            .set(expr)
            .execute(conn)
            .map(drop)
    }
}

impl QuestionStats {
    pub fn correct_ratio(&self) -> u8 {
        (self.num_correct as f32)
            .div(self.num_incorrect
                .add(self.num_correct)
                .max(1)
                as f32
            ).mul(100.)
            as u8
    }
}

impl Score {
    pub fn top_three(conn: &PgConnection) -> QueryResult<Vec<Score>> {
        use schema::scores::dsl::*;

        scores
            .order(weighted_points.desc())
            .limit(3)
            .load(conn)
    }

    pub fn placement(&self, conn: &PgConnection) -> QueryResult<u64> {
        use schema::scores::dsl::*;

        scores
            .filter(weighted_points.gt(self.points))
            .count()
            .get_result::<i64>(conn)
            .map(|count| count as u64 + 1)
    }

    pub fn neighbours(&self, conn: &PgConnection) -> QueryResult<(Vec<Score>, Vec<Score>)> {
        use schema::scores::dsl::*;

        let higher = scores
            .order(weighted_points.desc())
            .filter(weighted_points.gt(self.weighted_points))
            .limit(10)
            .load(conn)?;

        let lower = scores
            .order(weighted_points.desc())
            .filter(weighted_points.lt(self.weighted_points))
            .limit(10)
            .load(conn)?;

        Ok((higher, lower))
    }
}

#[derive(Debug)]
pub enum AdminError {
    Query(diesel::result::Error),
    NameInUse
}

impl Admin {
    pub fn named(named: &str, conn: &PgConnection) -> QueryResult<Option<Admin>> {
        use schema::admins::dsl::*;

        admins
            .filter(name.eq(named))
            .first(conn)
            .optional()
    }

    pub fn insert(new: &NewAdmin, conn: &PgConnection) -> Result<Admin, AdminError> {
        use schema::admins::dsl::*;

        let name_exists = Admin::named(new.name, conn)
            .map_err(AdminError::Query)?
            .is_some();

        if name_exists {
            Err(AdminError::NameInUse)
        } else {
            insert_into(admins)
                .values(new)
                .get_result(conn)
                .map_err(AdminError::Query)
        }
    }
}

impl <'a> NewQuestion<'a> {
    pub fn with_category(
        category: &Category,
        string: &'a str,
        correct: &'a str,
        incorrect: &'a [String]
    ) -> NewQuestion<'a> {
        NewQuestion {
            category_id: category.id,
            string,
            correct,
            incorrect
        }
    }
}

macro_rules! impl_insert {
    ($vis:vis, $name:ident, $new:ident, $table:ident) => {
        impl $name {
            $vis fn insert(new: &$new, conn: &PgConnection) -> QueryResult<$name> {
                use schema::$table::dsl::*;

                insert_into($table)
                    .values(new)
                    .get_result(conn)
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