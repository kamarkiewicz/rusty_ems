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

CREATE TABLE talks (
	id serial PRIMARY KEY,
	talk varchar NOT NULL UNIQUE,
	status smallint NOT NULL,
	title varchar NOT NULL,
	speaker_id integer NOT NULL REFERENCES persons (id),
	event_id integer REFERENCES events (id),
	room varchar,
	start_timestamp timestamp NOT NULL,
	add_timestamp timestamp NOT NULL DEFAULT now()
);

CREATE TABLE person_knows_person (
	person1_id integer REFERENCES persons (id),
	person2_id integer REFERENCES persons (id),
	PRIMARY KEY (person1_id, person2_id)
);

CREATE TABLE person_registered_for_event (
	person_id integer REFERENCES persons (id),
	event_id integer REFERENCES events (id),
	PRIMARY KEY (person_id, event_id)
);

CREATE TABLE person_attended_for_talk (
	person_id integer REFERENCES persons (id),
	talk_id integer REFERENCES talks (id),
	PRIMARY KEY (person_id, talk_id)
);

CREATE TABLE person_rated_talk (
	person_id integer REFERENCES persons (id),
	talk_id integer REFERENCES talks (id),
	rating smallint,
	PRIMARY KEY (person_id, talk_id)
);
