-- Set `ipid` to NULL when creating an entry to auto-generate an IPID
-- When deleting an IPID, all bans and logs entries containing that
-- IPID will also be deleted to fully erase the identity of a player.
CREATE TABLE IF NOT EXISTS ipids(
	ipid INTEGER PRIMARY KEY,
	ip_address TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS hdids(
	hdid TEXT PRIMARY KEY,
	ipid INTEGER NOT NULL,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE SET NULL,
	UNIQUE (hdid, ipid)
);

CREATE TABLE IF NOT EXISTS bans(
   ban_id INTEGER PRIMARY KEY,
   ban_date date DEFAULT CURRENT_TIMESTAMP,
   unban_date date,
   banned_by INTEGER,
   reason text,
   FOREIGN KEY (banned_by) REFERENCES ipids(ipid)
       ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS ip_bans(
	ipid INTEGER PRIMARY KEY,
	ban_id INTEGER NOT NULL,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE,
	FOREIGN KEY (ban_id) REFERENCES bans(ban_id)
		ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS hdid_bans(
	hdid TEXT PRIMARY KEY,
	ban_id INTEGER NOT NULL,
	FOREIGN KEY (hdid) REFERENCES hdids(hdid)
		ON UPDATE CASCADE
		ON DELETE CASCADE,
	FOREIGN KEY (ban_id) REFERENCES bans(ban_id)
		ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ic_events(
	event_time date DEFAULT CURRENT_TIMESTAMP,
	ipid integer NOT NULL,
	room_name text,
	char_name text,
	ic_name text,
	message text NOT NULL,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS room_event_types(
	type_id integer PRIMARY KEY,
	type_name text NOT NULL UNIQUE
);

/*
INSERT INTO room_event_types(type_name) VALUES
	('ooc'),
	('wtce'),
	('penalty'),
	('roll'),
	('notecard'),
	('notecard_reveal'),
	('rolla'),
	('coinflip'),
	('blockdj'),
	('unblockdj'),
	('disemvowel'),
	('undisemvowel'),
	('shake'),
	('unshake');
*/

-- Useful for RP events and announcements, not just chat
CREATE TABLE IF NOT EXISTS room_events(
	event_id integer PRIMARY KEY,
	event_time date DEFAULT CURRENT_TIMESTAMP,
	ipid integer NOT NULL,
	target_ipid integer,
	room_name text,
	char_name text,
	ooc_name text,
	event_subtype integer NOT NULL,
	message text,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE,
	FOREIGN KEY (target_ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE,
	FOREIGN KEY (event_subtype) REFERENCES room_event_types(type_id)
);

-- `profile_name` is NULL if the login attempt failed
CREATE TABLE IF NOT EXISTS login_events(
	event_time date DEFAULT CURRENT_TIMESTAMP,
	ipid INTEGER NOT NULL,
	profile_name TEXT,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS connect_events(
	event_time date DEFAULT CURRENT_TIMESTAMP,
	ipid INTEGER NOT NULL,
	hdid TEXT NOT NULL,
	failed INTEGER DEFAULT 0,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS misc_event_types(
	type_id INTEGER PRIMARY KEY,
	type_name TEXT NOT NULL UNIQUE
);

/*
INSERT INTO misc_event_types(type_name) VALUES
	('system'), -- server start, stop, reload
	('kick'),
	('ban'),
	('unban');
*/

-- Useful for system, admin, and user-defined events
CREATE TABLE IF NOT EXISTS misc_events(
	event_time date DEFAULT CURRENT_TIMESTAMP,
	ipid INTEGER,
	target_ipid INTEGER,
	event_subtype INTEGER NOT NULL,
	event_data TEXT,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE,
	FOREIGN KEY (target_ipid) REFERENCES ipids(ipid)
		ON DELETE CASCADE,
	FOREIGN KEY (event_subtype) REFERENCES misc_event_types(type_id)
);

CREATE TABLE IF NOT EXISTS general_info(
    db_version integer NOT NULL
);

INSERT INTO general_info (db_version) VALUES (1);