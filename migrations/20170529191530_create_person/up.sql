CREATE TABLE person (
  id serial PRIMARY KEY,
  login varchar NOT NULL UNIQUE,
  password varchar NOT NULL,
  is_organizer boolean NOT NULL DEFAULT false
);
