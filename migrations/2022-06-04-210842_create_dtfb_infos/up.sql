CREATE TABLE dtfb_player_ids (
	itsf_id INTEGER PRIMARY KEY NOT NULL,
    dtfb_id INTEGER NOT NULL
);

CREATE TABLE dtfb_player_teams (
	player_itsf_id INTEGER NOT NULL,
    year INTEGER NOT NULL,
	team_name TEXT NOT NULL,
    PRIMARY KEY(player_itsf_id, year)
);

CREATE TABLE dtfb_national_championship_results (
	player_itsf_id INTEGER NOT NULL,
    year INTEGER NOT NULL,
    place INTEGER NOT NULL,
    category INTEGER NOT NULL,
    class INTEGER NOT NULL,
    PRIMARY KEY(player_itsf_id, year)
);

CREATE TABLE dtfb_national_rankings (
	player_itsf_id INTEGER NOT NULL,
    year INTEGER NOT NULL,
    place INTEGER NOT NULL,
    category INTEGER NOT NULL,
    PRIMARY KEY(player_itsf_id, year)
);
