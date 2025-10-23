# Current Features

1. Story/Fragment/Location/Character Creation.
2. Image uploads.
3. Authentication (Local, and OIDC).
4. Content Tagging (with EXIF + custom entity tags).

# Future Features

1. Fragment merging (by date range, manually, or based on graph structure).
2. Manual graph creation.
3. Automated graph generation for heterogeneous graphs.
4. XMP + IPTC Deserialize, and Serialization for NSFW/Content Warnings.

---

## Relationships

Relationships model the connections between narrative entities in this system. Below is a diagram representing the current relationship hierarchy:

```
Story
├── Fragment
│   ├── Subfragment         (fragment → fragment)
│   ├── Character           (mentions)
│   └── Location            (occurs_at)
├── Character
│   ├── Location            (resides_in / visits)
│   ├── Fragment            (appears_in)
│   └── Character           (subcharacter_of)
├── Location
│   ├── Fragment            (contains)
│   └── Location            (sublocation_of)
├── Story                  (substory / reference)
├── Pattern
│   ├── Fragment            (pattern_applies_to)
│   ├── Character           (exhibits_pattern)
│   └── Location            (pattern_occurs_at)
└── Timeline
    └── Fragment            (temporally indexes)
```

<details>
<summary>🗂 Legend</summary>

| Label         | Source → Target         | Description                                         |
|---------------|-------------------------|-----------------------------------------------------|
| `fragment`    | Story → Fragment        | Story contains this narrative fragment              |
| `subfragment` | Fragment → Fragment     | Fragment contains a nested scene or subfragment     |
| `mentions`    | Fragment → Character    | Character is involved in the fragment               |
| `occurs_at`   | Fragment → Location     | Fragment takes place at the specified location      |
| `character`   | Story → Character       | Character appears in the story                      |
| `location`    | Story → Location        | Location is relevant to the story                   |
| `resides_in`  | Character → Location    | Character resides in or is associated with location |
| `appears_in`  | Character → Fragment    | Character appears in the scene                      |
| `contains`    | Location → Fragment     | Location contains the given fragment                |
| `substory`    | Story → Story           | A nested or referenced substory                     |

</details>

---

### Notes on Hierarchy

- A **story** is the root entity (unless working with a larger narrative universe).
- A **story** can contain:
  - Fragments (scenes),
  - Characters,
  - Locations,
  - Substories.
- A **fragment** can include:
  - Subfragments (nested scenes),
  - Characters involved,
  - A location where the scene takes place.
- A **character** can:
  - Appear in fragments,
  - Be associated with specific locations.
- A **location** can:
  - Contain fragments occurring at that place.

While stories form the root of the hierarchy, other entities (fragments, characters, locations) may form recursive or cyclical structures where appropriate.
