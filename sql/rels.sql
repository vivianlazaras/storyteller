-- Create or replace table to link characters to fragments
CREATE TABLE characterfragments (
    character UUID NOT NULL,
    fragment UUID NOT NULL,
    PRIMARY KEY (character, fragment),
    FOREIGN KEY (character) REFERENCES characters(id) ON DELETE CASCADE,
    FOREIGN KEY (fragment) REFERENCES fragments(id) ON DELETE CASCADE
);

-- Create or replace table to link locations to fragments
CREATE TABLE locationfragments (
    location UUID NOT NULL,
    fragment UUID NOT NULL,
    PRIMARY KEY (location, fragment),
    FOREIGN KEY (location) REFERENCES locations(id) ON DELETE CASCADE,
    FOREIGN KEY (fragment) REFERENCES fragments(id) ON DELETE CASCADE
);

-- Create or replace table to link stories to fragments
CREATE TABLE storyfragments (
    story UUID NOT NULL,
    fragment UUID NOT NULL,
    PRIMARY KEY (story, fragment),
    FOREIGN KEY (story) REFERENCES stories(id) ON DELETE CASCADE,
    FOREIGN KEY (fragment) REFERENCES fragments(id) ON DELETE CASCADE
);