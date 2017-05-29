
-- -- object: rusty_ems | type: DATABASE --
-- -- DROP DATABASE IF EXISTS rusty_ems;
-- CREATE DATABASE rusty_ems;

-- object: user | type: TABLE --
-- DROP TABLE IF EXISTS user CASCADE;
CREATE TABLE user(
	user_id bigserial NOT NULL,
	login varchar(32) NOT NULL,
	password text NOT NULL,
	is_organizer boolean NOT NULL DEFAULT false,
	CONSTRAINT user_pk PRIMARY KEY (user_id)

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
	user_id bigint,
	event_id bigint,
	start_timestamp timestamp NOT NULL,
	add_timestamp timestamp NOT NULL DEFAULT now(),
	CONSTRAINT talk_pk PRIMARY KEY (talk_id)

);

-- object: user_knows_user | type: TABLE --
-- DROP TABLE IF EXISTS user_knows_user CASCADE;
CREATE TABLE user_knows_user(
	user1_id bigint NOT NULL,
	user2_id bigint NOT NULL,
	CONSTRAINT "user_knows_user_pk" PRIMARY KEY (user1_id,user2_id)

);

-- object: user_attended_for_talk | type: TABLE --
-- DROP TABLE IF EXISTS user_attended_for_talk CASCADE;
CREATE TABLE user_attended_for_talk(
	user_id bigint,
	talk_id text,
	present boolean NOT NULL DEFAULT false,
	rating smallint,
	CONSTRAINT "User_attended_for_Talk_pk" PRIMARY KEY (user_id,talk_id)

);

-- object: user_fk | type: CONSTRAINT --
-- ALTER TABLE user_attended_for_talk DROP CONSTRAINT IF EXISTS user_fk CASCADE;
ALTER TABLE user_attended_for_talk ADD CONSTRAINT user_fk FOREIGN KEY (user_id)
REFERENCES user (user_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: talk_fk | type: CONSTRAINT --
-- ALTER TABLE user_attended_for_talk DROP CONSTRAINT IF EXISTS talk_fk CASCADE;
ALTER TABLE user_attended_for_talk ADD CONSTRAINT talk_fk FOREIGN KEY (talk_id)
REFERENCES talk (talk_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: event_fk | type: CONSTRAINT --
-- ALTER TABLE talk DROP CONSTRAINT IF EXISTS event_fk CASCADE;
ALTER TABLE talk ADD CONSTRAINT event_fk FOREIGN KEY (event_id)
REFERENCES event (event_id) MATCH FULL
ON DELETE SET NULL ON UPDATE CASCADE;

-- object: user_fk | type: CONSTRAINT --
-- ALTER TABLE talk DROP CONSTRAINT IF EXISTS user_fk CASCADE;
ALTER TABLE talk ADD CONSTRAINT user_fk FOREIGN KEY (user_id)
REFERENCES user (user_id) MATCH FULL
ON DELETE SET NULL ON UPDATE CASCADE;

-- object: user_registered_for_event | type: TABLE --
-- DROP TABLE IF EXISTS user_registered_for_event CASCADE;
CREATE TABLE user_registered_for_event(
	user_id bigint,
	event_id bigint,
	CONSTRAINT "User_registered_for_Event_pk" PRIMARY KEY (user_id,event_id)

);

-- object: user_fk | type: CONSTRAINT --
-- ALTER TABLE user_registered_for_event DROP CONSTRAINT IF EXISTS user_fk CASCADE;
ALTER TABLE user_registered_for_event ADD CONSTRAINT user_fk FOREIGN KEY (user_id)
REFERENCES user (user_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: event_fk | type: CONSTRAINT --
-- ALTER TABLE user_registered_for_event DROP CONSTRAINT IF EXISTS event_fk CASCADE;
ALTER TABLE user_registered_for_event ADD CONSTRAINT event_fk FOREIGN KEY (event_id)
REFERENCES event (event_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;

-- object: user1_knows_user2 | type: CONSTRAINT --
-- ALTER TABLE user_knows_user DROP CONSTRAINT IF EXISTS user1_knows_user2 CASCADE;
ALTER TABLE user_knows_user ADD CONSTRAINT user1_knows_user2 FOREIGN KEY (user1_id)
REFERENCES user (user_id) MATCH FULL
ON DELETE NO ACTION ON UPDATE NO ACTION;

-- object: user2_knows_user1 | type: CONSTRAINT --
-- ALTER TABLE user_knows_user DROP CONSTRAINT IF EXISTS user2_knows_user1 CASCADE;
ALTER TABLE user_knows_user ADD CONSTRAINT user2_knows_user1 FOREIGN KEY (user2_id)
REFERENCES user (user_id) MATCH FULL
ON DELETE NO ACTION ON UPDATE NO ACTION;
