-- Add migration script here
CREATE TABLE IF NOT EXISTS urls (
    id serial primary key,
    real_url varchar not null,
    generated_url varchar not null
);
