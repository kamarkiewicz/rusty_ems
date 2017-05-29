
-- -- object: rusty_ems | type: DATABASE --
-- -- DROP DATABASE IF EXISTS rusty_ems;
-- CREATE DATABASE rusty_ems;

-- object: person | type: TABLE --
-- DROP TABLE IF EXISTS person CASCADE;
CREATE TABLE person(
	person_id bigserial NOT NULL,
	login varchar(32) NOT NULL,
	password text NOT NULL,
	is_organizer boolean NOT NULL DEFAULT false,
	CONSTRAINT person_pk PRIMARY KEY (person_id)
);

-- object: event | type: TABLE --
-- DROP TABLE IF EXISTS event CASCADE;
CREATE TABLE event(
	event_id bigint NOT NULL,
	eventname text NOT NULL,
	start_timestamp timestamp NOT NULL,
	end_timestamp timestamp NOT NULL,
	CONSTRAINT event_pk PRIMARY KEY (event_id)
);

-- object: talk | type: TABLE --
-- DROP TABLE IF EXISTS talk CASCADE;
CREATE TABLE talk(
	talk_id text NOT NULL,
	status smallint NOT NULL,
	title text NOT NULL,
	person_id bigint,
	event_id bigint,
	start_timestamp timestamp NOT NULL,
	add_timestamp timestamp NOT NULL DEFAULT now(),
	CONSTRAINT talk_pk PRIMARY KEY (talk_id)
);

-- object: person_knows_person | type: TABLE --
-- DROP TABLE IF EXISTS person_knows_person CASCADE;
CREATE TABLE person_knows_person(
	person1_id bigint NOT NULL,
	person2_id bigint NOT NULL,
	CONSTRAINT "person_knows_person_pk" PRIMARY KEY (person1_id,person2_id)
);

-- object: person_attended_for_talk | type: TABLE --
-- DROP TABLE IF EXISTS person_attended_for_talk CASCADE;
CREATE TABLE person_attended_for_talk(
	person_id bigint,
	talk_id text,
	present boolean NOT NULL DEFAULT false,
	rating smallint,
	CONSTRAINT "person_attended_for_Talk_pk" PRIMARY KEY (person_id,talk_id)
);

-- object: person_fk | type: CONSTRAINT --
-- ALTER TABLE person_attended_for_talk DROP CONSTRAINT IF EXISTS person_fk CASCADE;
ALTER TABLE person_attended_for_talk ADD CONSTRAINT person_fk FOREIGN KEY (person_id)
REFERENCES person (person_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: talk_fk | type: CONSTRAINT --
-- ALTER TABLE person_attended_for_talk DROP CONSTRAINT IF EXISTS talk_fk CASCADE;
ALTER TABLE person_attended_for_talk ADD CONSTRAINT talk_fk FOREIGN KEY (talk_id)
REFERENCES talk (talk_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: event_fk | type: CONSTRAINT --
-- ALTER TABLE talk DROP CONSTRAINT IF EXISTS event_fk CASCADE;
ALTER TABLE talk ADD CONSTRAINT event_fk FOREIGN KEY (event_id)
REFERENCES event (event_id) MATCH FULL
ON DELETE SET NULL ON UPDATE CASCADE;

-- object: person_fk | type: CONSTRAINT --
-- ALTER TABLE talk DROP CONSTRAINT IF EXISTS person_fk CASCADE;
ALTER TABLE talk ADD CONSTRAINT person_fk FOREIGN KEY (person_id)
REFERENCES person (person_id) MATCH FULL
ON DELETE SET NULL ON UPDATE CASCADE;

-- object: person_registered_for_event | type: TABLE --
-- DROP TABLE IF EXISTS person_registered_for_event CASCADE;
CREATE TABLE person_registered_for_event(
	person_id bigint,
	event_id bigint,
	CONSTRAINT "person_registered_for_Event_pk" PRIMARY KEY (person_id,event_id)
);

-- object: person_fk | type: CONSTRAINT --
-- ALTER TABLE person_registered_for_event DROP CONSTRAINT IF EXISTS person_fk CASCADE;
ALTER TABLE person_registered_for_event ADD CONSTRAINT person_fk FOREIGN KEY (person_id)
REFERENCES person (person_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: event_fk | type: CONSTRAINT --
-- ALTER TABLE person_registered_for_event DROP CONSTRAINT IF EXISTS event_fk CASCADE;
ALTER TABLE person_registered_for_event ADD CONSTRAINT event_fk FOREIGN KEY (event_id)
REFERENCES event (event_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: person1_knows_person2 | type: CONSTRAINT --
-- ALTER TABLE person_knows_person DROP CONSTRAINT IF EXISTS person1_knows_person2 CASCADE;
ALTER TABLE person_knows_person ADD CONSTRAINT person1_knows_person2 FOREIGN KEY (person1_id)
REFERENCES person (person_id) MATCH FULL
ON DELETE NO ACTION ON UPDATE NO ACTION;

-- object: person2_knows_person1 | type: CONSTRAINT --
-- ALTER TABLE person_knows_person DROP CONSTRAINT IF EXISTS person2_knows_person1 CASCADE;
ALTER TABLE person_knows_person ADD CONSTRAINT person2_knows_person1 FOREIGN KEY (person2_id)
REFERENCES person (person_id) MATCH FULL
ON DELETE NO ACTION ON UPDATE NO ACTION;
