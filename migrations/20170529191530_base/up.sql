CREATE TABLE persons (
  id serial PRIMARY KEY,
  login varchar NOT NULL UNIQUE,
  password varchar NOT NULL,
  is_organizer boolean NOT NULL DEFAULT false
);

CREATE TABLE events (
	id serial PRIMARY KEY,
	eventname varchar NOT NULL UNIQUE,
	start_timestamp timestamp NOT NULL,
	end_timestamp timestamp NOT NULL
);
