CREATE TABLE players (
	itsf_id INTEGER PRIMARY KEY NOT NULL,
	first_name TEXT NOT NULL,
	last_name TEXT NOT NULL,
	birth_year INTEGER NOT NULL,
	country_code CHAR(10),
	category INTEGER NOT NULL
);

CREATE TABLE player_images (
	itsf_id INTEGER PRIMARY KEY NOT NULL,
	image_data BLOB NOT NULL,
	image_format CHAR(8)
);
