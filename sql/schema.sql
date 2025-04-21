CREATE TABLE IF NOT EXISTS license (
    id uuid primary key,
    name text not null,
    description text,
    content text,
);



CREATE TABLE IF NOT EXISTS Stories (
    id uuid primary key,
    name text not null,
    description text not null,
    url text not null,
    license uuid references license(id),
);