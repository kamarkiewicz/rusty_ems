CREATE TABLE event (
	id serial PRIMARY KEY,
	eventname varchar NOT NULL UNIQUE,
	start_timestamp timestamp NOT NULL,
	end_timestamp timestamp NOT NULL
);
