-- Your SQL goes here
CREATE TABLE tests (
    id SERIAL PRIMARY KEY,
    name VARCHAR(127) NOT NULL,
    description TEXT,
    instructions TEXT,
    logo VARCHAR(511),
    points_reference JSONB NOT NULL,
    points_interpretation JSONB NOT NULL
);
