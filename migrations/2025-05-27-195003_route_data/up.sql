CREATE TABLE IF NOT EXISTS route_data (
    id SERIAL NOT NULL PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    visibility INT2 NOT NULL, -- 0 = public, 1 = hidden, 2 = private (only authenticated tokens)
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
