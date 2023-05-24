-- Add migration script here
CREATE TABLE "user"(
    id SERIAL NOT NULL,
    name VARCHAR(255) NOT NULL UNIQUE
);
