table! {
    admins (id) {
        id -> Int4,
        name -> Text,
        password -> Text,
    }
}

table! {
    categories (id) {
        id -> Int4,
        name -> Text,
    }
}

table! {
    question_stats (id) {
        id -> Int4,
        question_id -> Int4,
        num_correct -> Int4,
        num_incorrect -> Int4,
    }
}

table! {
    questions (id) {
        id -> Int4,
        category_id -> Int4,
        string -> Text,
        correct -> Text,
        incorrect -> Array<Text>,
    }
}

table! {
    scores (id) {
        id -> Int4,
        name -> Text,
        points -> Int4,
    }
}

joinable!(question_stats -> questions (question_id));
joinable!(questions -> categories (category_id));

allow_tables_to_appear_in_same_query!(
    admins,
    categories,
    question_stats,
    questions,
    scores,
);
