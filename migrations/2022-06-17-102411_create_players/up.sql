CREATE TABLE players (
	itsf_id INTEGER PRIMARY KEY NOT NULL,
	json_data BLOB NOT NULL
);

CREATE TABLE player_images (
	itsf_id INTEGER PRIMARY KEY NOT NULL,
	image_data BLOB NOT NULL,
	image_format CHAR(8) NOT NULL
);
