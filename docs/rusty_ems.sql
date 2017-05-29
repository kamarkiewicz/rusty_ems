-- Database generated with pgModeler (PostgreSQL Database Modeler).
-- pgModeler  version: 0.8.2
-- PostgreSQL version: 9.5
-- Project Site: pgmodeler.com.br
-- Model Author: ---


-- Database creation must be done outside an multicommand file.
-- These commands were put in this file only for convenience.
-- -- object: rusty_ems | type: DATABASE --
-- -- DROP DATABASE IF EXISTS rusty_ems;
-- CREATE DATABASE rusty_ems
-- ;
-- -- ddl-end --
-- 

-- object: public."User" | type: TABLE --
-- DROP TABLE IF EXISTS public."User" CASCADE;
CREATE TABLE public."User"(
	user_id bigserial NOT NULL,
	login varchar(32) NOT NULL,
	password text NOT NULL,
	is_organizer boolean NOT NULL DEFAULT false,
	CONSTRAINT user_pk PRIMARY KEY (user_id)

);
-- ddl-end --
ALTER TABLE public."User" OWNER TO postgres;
-- ddl-end --

-- object: public."Event" | type: TABLE --
-- DROP TABLE IF EXISTS public."Event" CASCADE;
CREATE TABLE public."Event"(
	event_id bigint NOT NULL,
	eventname text NOT NULL,
	start_timestamp timestamp NOT NULL,
	end_timestamp timestamp NOT NULL,
	CONSTRAINT event_pk PRIMARY KEY (event_id)

);
-- ddl-end --
ALTER TABLE public."Event" OWNER TO postgres;
-- ddl-end --

-- object: public."Talk" | type: TABLE --
-- DROP TABLE IF EXISTS public."Talk" CASCADE;
CREATE TABLE public."Talk"(
	talk_id text NOT NULL,
	status smallint NOT NULL,
	title text NOT NULL,
	"user_id_User" bigint,
	"event_id_Event" bigint,
	start_timestamp timestamp NOT NULL,
	add_timestamp timestamp NOT NULL DEFAULT now(),
	CONSTRAINT talk_pk PRIMARY KEY (talk_id)

);
-- ddl-end --
ALTER TABLE public."Talk" OWNER TO postgres;
-- ddl-end --

-- object: public."User_friend_of_User" | type: TABLE --
-- DROP TABLE IF EXISTS public."User_friend_of_User" CASCADE;
CREATE TABLE public."User_friend_of_User"(
	user1_id bigint NOT NULL,
	user2_id bigint NOT NULL,
	CONSTRAINT "User_friend_of_User_pk" PRIMARY KEY (user1_id,user2_id)

);
-- ddl-end --
ALTER TABLE public."User_friend_of_User" OWNER TO postgres;
-- ddl-end --

-- object: public."User_attended_for_Talk" | type: TABLE --
-- DROP TABLE IF EXISTS public."User_attended_for_Talk" CASCADE;
CREATE TABLE public."User_attended_for_Talk"(
	"user_id_User" bigint,
	"talk_id_Talk" text,
	present boolean NOT NULL DEFAULT false,
	rating smallint,
	CONSTRAINT "User_attended_for_Talk_pk" PRIMARY KEY ("user_id_User","talk_id_Talk")

);
-- ddl-end --

-- object: "User_fk" | type: CONSTRAINT --
-- ALTER TABLE public."User_attended_for_Talk" DROP CONSTRAINT IF EXISTS "User_fk" CASCADE;
ALTER TABLE public."User_attended_for_Talk" ADD CONSTRAINT "User_fk" FOREIGN KEY ("user_id_User")
REFERENCES public."User" (user_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: "Talk_fk" | type: CONSTRAINT --
-- ALTER TABLE public."User_attended_for_Talk" DROP CONSTRAINT IF EXISTS "Talk_fk" CASCADE;
ALTER TABLE public."User_attended_for_Talk" ADD CONSTRAINT "Talk_fk" FOREIGN KEY ("talk_id_Talk")
REFERENCES public."Talk" (talk_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: "Event_fk" | type: CONSTRAINT --
-- ALTER TABLE public."Talk" DROP CONSTRAINT IF EXISTS "Event_fk" CASCADE;
ALTER TABLE public."Talk" ADD CONSTRAINT "Event_fk" FOREIGN KEY ("event_id_Event")
REFERENCES public."Event" (event_id) MATCH FULL
ON DELETE SET NULL ON UPDATE CASCADE;
-- ddl-end --

-- object: "User_fk" | type: CONSTRAINT --
-- ALTER TABLE public."Talk" DROP CONSTRAINT IF EXISTS "User_fk" CASCADE;
ALTER TABLE public."Talk" ADD CONSTRAINT "User_fk" FOREIGN KEY ("user_id_User")
REFERENCES public."User" (user_id) MATCH FULL
ON DELETE SET NULL ON UPDATE CASCADE;
-- ddl-end --

-- object: public."User_registered_for_Event" | type: TABLE --
-- DROP TABLE IF EXISTS public."User_registered_for_Event" CASCADE;
CREATE TABLE public."User_registered_for_Event"(
	"user_id_User" bigint,
	"event_id_Event" bigint,
	CONSTRAINT "User_registered_for_Event_pk" PRIMARY KEY ("user_id_User","event_id_Event")

);
-- ddl-end --

-- object: "User_fk" | type: CONSTRAINT --
-- ALTER TABLE public."User_registered_for_Event" DROP CONSTRAINT IF EXISTS "User_fk" CASCADE;
ALTER TABLE public."User_registered_for_Event" ADD CONSTRAINT "User_fk" FOREIGN KEY ("user_id_User")
REFERENCES public."User" (user_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: "Event_fk" | type: CONSTRAINT --
-- ALTER TABLE public."User_registered_for_Event" DROP CONSTRAINT IF EXISTS "Event_fk" CASCADE;
ALTER TABLE public."User_registered_for_Event" ADD CONSTRAINT "Event_fk" FOREIGN KEY ("event_id_Event")
REFERENCES public."Event" (event_id) MATCH FULL
ON DELETE CASCADE ON UPDATE CASCADE;
-- ddl-end --

-- object: user1_friend_of_user2 | type: CONSTRAINT --
-- ALTER TABLE public."User_friend_of_User" DROP CONSTRAINT IF EXISTS user1_friend_of_user2 CASCADE;
ALTER TABLE public."User_friend_of_User" ADD CONSTRAINT user1_friend_of_user2 FOREIGN KEY (user1_id)
REFERENCES public."User" (user_id) MATCH FULL
ON DELETE NO ACTION ON UPDATE NO ACTION;
-- ddl-end --

-- object: user2_friend_of_user1 | type: CONSTRAINT --
-- ALTER TABLE public."User_friend_of_User" DROP CONSTRAINT IF EXISTS user2_friend_of_user1 CASCADE;
ALTER TABLE public."User_friend_of_User" ADD CONSTRAINT user2_friend_of_user1 FOREIGN KEY (user2_id)
REFERENCES public."User" (user_id) MATCH FULL
ON DELETE NO ACTION ON UPDATE NO ACTION;
-- ddl-end --


