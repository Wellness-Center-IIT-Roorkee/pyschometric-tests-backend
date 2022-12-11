-- Your SQL goes here
CREATE TABLE questions (
    id SERIAL PRIMARY KEY,
    test_id SERIAL NOT NULL,
    text TEXT NOT NULL,
    CONSTRAINT fk_test
        FOREIGN KEY(test_id)
            REFERENCES tests(id)
            ON DELETE CASCADE
);