-- Add migration script here
CREATE TYPE gender AS ENUM ('female', 'male', 'unknown');

CREATE TABLE user_stats (
    email VARCHAR(128) NOT NULL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    gender gender default 'unknown',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_visited_at TIMESTAMPTZ,
    last_watched_at TIMESTAMPTZ,
    recent_watched INT[],
    viewed_but_not_started INT[],
    started_but_not_finished INT[],
    finished INT[],
    last_email_notification TIMESTAMPTZ,
    last_in_app_notification TIMESTAMPTZ,
    last_sms_notification TIMESTAMPTZ
);
