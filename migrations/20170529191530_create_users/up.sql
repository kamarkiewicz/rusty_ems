CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  login VARCHAR NOT NULL,
  password VARCHAR NOT NULL,
  is_organizer BOOLEAN NOT NULL DEFAULT FALSE
)
