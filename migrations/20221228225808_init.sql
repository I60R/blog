CREATE DATABASE blogs;

CREATE TABLE IF NOT EXISTS blogs (
    id INTEGER UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    added TEXT NOT NULL
);
