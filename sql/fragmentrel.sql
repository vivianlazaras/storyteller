CREATE TABLE storyfragments (
    story UUID NOT NULL,
    fragment UUID NOT NULL,
    PRIMARY KEY (story, fragment),
    FOREIGN KEY (story) REFERENCES stories(id) ON DELETE CASCADE,
    FOREIGN KEY (fragment) REFERENCES fragments(id) ON DELETE CASCADE
);