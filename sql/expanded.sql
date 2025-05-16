CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    fname text NOT NULL,
    lname text NOT NULL,
    subject uuid,
    email text NOT NULL
);

CREATE TABLE IF NOT EXISTS usergroups (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text,
    description text
);

CREATE TABLE IF NOT EXISTS licenses (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    description text,
    public boolean DEFAULT false,
    content text
);

CREATE TABLE IF NOT EXISTS grouprel (
    id uuid PRIMARY KEY,
    group_id uuid REFERENCES groups(id),
    user_id uuid REFERENCES users(id),
    description text
);

CREATE TABLE IF NOT EXISTS universe (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    description text,
    
    -- Metadata fields
    created bigint,
    last_edited bigint,
    creator uuid REFERENCES users(id),
    license uuid REFERENCES licenses(id),
    shared uuid REFERENCES groups(id)
);

CREATE TABLE IF NOT EXISTS timelines (
    id uuid PRIMARY KEY,

    -- Metadata fields
    created bigint,
    last_edited bigint,
    creator uuid REFERENCES users(id),
    license uuid REFERENCES licenses(id),
    shared uuid REFERENCES groups(id)
);

CREATE TABLE IF NOT EXISTS characters (
    id uuid PRIMARY KEY,
    timeline uuid REFERENCES timelines(id),
    name text NOT NULL,
    description text,

    -- Metadata fields
    created bigint,
    last_edited bigint,
    creator uuid REFERENCES users(id),
    license uuid REFERENCES licenses(id),
    shared uuid REFERENCES groups(id),
    public boolean default false
);

CREATE TABLE IF NOT EXISTS stories (
    id uuid PRIMARY KEY,
    timeline uuid REFERENCES timelines(id),
    name text NOT NULL,
    description text,
    renderer text,
    content bytea,

    -- Metadata fields
    created bigint,
    last_edited bigint,
    creator uuid REFERENCES users(id),
    license uuid REFERENCES licenses(id),
    shared uuid REFERENCES groups(id),
    public boolean default false
);

CREATE TABLE IF NOT EXISTS tags (
    id uuid PRIMARY KEY,
    story uuid REFERENCES stories(id),
    value text NOT NULL
);

CREATE TABLE IF NOT EXISTS characterrel (
    id uuid PRIMARY KEY,
    description text,
    character uuid REFERENCES characters(id),
    story uuid REFERENCES stories(id)
);