-- Just checks for existance of required tables in database
SELECT 'persons'::regclass,
       'events'::regclass,
       'talks'::regclass,
       'person_knows_person'::regclass,
       'person_registered_for_event'::regclass,
       'person_attended_for_talk'::regclass,
       'person_rated_talk'::regclass;
