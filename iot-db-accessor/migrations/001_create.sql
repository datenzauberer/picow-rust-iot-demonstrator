CREATE TABLE IF NOT EXISTS sensor_values (
    id        INTEGER PRIMARY KEY NOT NULL,
    timestamp DATETIME DEFAULT(STRFTIME('%Y-%m-%d %H:%M:%f', 'NOW')),
    value     REAL
);
