-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    fname text NOT NULL,
    lname text NOT NULL,
    subject uuid,
    email text NOT NULL
);

-- Create licenses table
CREATE TABLE IF NOT EXISTS licenses (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    description text,
    public boolean DEFAULT false,
    content text
);

-- Create usergroups table
CREATE TABLE IF NOT EXISTS usergroups (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text,
    description text
);

-- Create metadata table
CREATE TABLE IF NOT EXISTS metadata (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    creator uuid REFERENCES users(id),
    license uuid REFERENCES licenses(id),
    shared uuid REFERENCES usergroups(id),
    public boolean DEFAULT false
);

-- Create grouprel table
CREATE TABLE IF NOT EXISTS grouprel (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    groupid uuid REFERENCES usergroups(id),
    userid uuid REFERENCES users(id),
    description text
);

-- Create universe table
CREATE TABLE IF NOT EXISTS universe (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    description text,
    metadata uuid REFERENCES metadata(id),
    created bigint,
    last_edited bigint
);

-- Create timelines table
CREATE TABLE IF NOT EXISTS timelines (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    metadata uuid REFERENCES metadata(id),
    created bigint,
    last_edited bigint
);

-- Create characters table
CREATE TABLE IF NOT EXISTS characters (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    timeline uuid REFERENCES timelines(id),
    name text NOT NULL,
    description text,
    metadata uuid REFERENCES metadata(id),
    created bigint,
    last_edited bigint
);

-- Create stories table
CREATE TABLE IF NOT EXISTS stories (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    timeline uuid REFERENCES timelines(id),
    name text NOT NULL,
    description text,
    renderer text,
    content bytea,
    metadata uuid REFERENCES metadata(id),
    created bigint,
    last_edited bigint
);

-- Create tags table
CREATE TABLE IF NOT EXISTS tags (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    story uuid REFERENCES stories(id),
    value text NOT NULL
);

-- Create characterrel table
CREATE TABLE IF NOT EXISTS characterrel (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    description text,
    character uuid REFERENCES characters(id),
    story uuid REFERENCES stories(id)
);