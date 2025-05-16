CREATE TABLE IF NOT EXISTS users (
    id uuid primary key default gen_random_uuid(),
    fname text not null,
    lname text not null,
    subject uuid,
    email text not null
);

CREATE TABLE IF NOT EXISTS groups (
    id uuid primary key default gen_random_uuid(),
    name text,
    description text
);

CREATE TABLE IF NOT EXISTS licenses (
    id uuid primary key,
    name text not null,
    description text,
    public boolean default false,
    content text
);

CREATE TABLE IF NOT EXISTS metadata (
    id uuid primary key,
    created bigint,
    last_edited bigint,
    creator uuid references users(id),
    license uuid references licenses(id),
    shared uuid references groups(id)
);

CREATE TABLE IF NOT EXISTS grouprel (
    id uuid primary key,
    group_id uuid references groups(id),
    user_id uuid references users(id),
    description text
);

CREATE TABLE IF NOT EXISTS universe (
    id uuid primary key,
    name text not null,
    description text,
);

CREATE TABLE IF NOT EXISTS timelines (
    id uuid primary key,
    metadata uuid references metadata(id)
);

CREATE TABLE IF NOT EXISTS characters (
    id uuid primary key,
    metadata uuid references metadata(id),
    timeline uuid references timelines(id),
    name text not null,
    description text
);

CREATE TABLE IF NOT EXISTS stories (
    id uuid primary key,
    metadata uuid references metadata(id),
    timeline uuid references timelines(id),
    name text not null,
    description text,
    content bytea
);

CREATE TABLE IF NOT EXISTS tags (
    id uuid primary key,
    story uuid references stories(id),
    value text not null
);

CREATE TABLE IF NOT EXISTS characterrel (
    id uuid primary key,
    description text,
    character uuid references characters(id),
    story uuid references stories(id)
);