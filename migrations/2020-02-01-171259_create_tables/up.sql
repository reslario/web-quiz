CREATE TABLE categories
(
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE questions
(
    id          SERIAL PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories (id),
    string      TEXT    NOT NULL,
    correct     TEXT    NOT NULL,
    incorrect   TEXT[3] NOT NULL
);

CREATE TABLE question_stats
(
    id            SERIAL PRIMARY KEY,
    question_id   INTEGER NOT NULL REFERENCES questions (id),
    num_correct   INTEGER NOT NULL DEFAULT 0,
    num_incorrect INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE scores
(
    id     SERIAL PRIMARY KEY,
    name   TEXT    NOT NULL,
    points INTEGER NOT NULL
);
