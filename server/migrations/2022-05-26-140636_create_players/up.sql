CREATE TABLE players (
	itsf_id INTEGER PRIMARY KEY NOT NULL,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	dtfb_license TEXT,
	birth_year INTEGER NOT NULL,
	country_code CHAR(10),
	category CHAR(10)
)
