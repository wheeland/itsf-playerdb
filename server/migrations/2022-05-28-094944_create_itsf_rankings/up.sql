CREATE TABLE itsf_rankings (
	id INTEGER PRIMARY KEY,
    year INTEGER NOT NULL,
	queried_at DATETIME NOT NULL,
	count INTEGER NOT NULL,
	category CHAR(10)
);

CREATE TABLE itsf_ranking_entries (
    itsf_ranking_id INTEGER NOT NULL,
    place INTEGER NOT NULL,
    player_itsf_id INTEGER NOT NULL,
    PRIMARY KEY(itsf_ranking_id, place)
);
