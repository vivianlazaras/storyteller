# Group-Based Permission System Design

This chapter outlines the design and implementation of a flexible, auditable, and performant group-based permission system that supports user-defined hierarchies, shared access, and secure delegation.

## Goals

- Make **groups first-class** entities users can understand and manage.
- Support **permission inheritance** via group hierarchies.
- Enable **secure, efficient checks** for access control.
- Allow for **shared access** across teams using hidden shared groups.
- Keep the system intuitive for technical and non-technical users.

---

## Core Concepts

### Groups

Groups are logical collections of users and entities. They are hierarchical (a tree), and permissions are inherited from ancestor groups.

- Groups can be **visible** (user-facing) or **hidden** (used for internal access logic).
- Every group is represented in a closure table to support recursive lookup.

### Permissions

Permissions are not stored on individual links or entities, but inferred through group relationships. The base access types include:

- `read`
- `write`
- `update`
- `delete`

Custom permissions may be defined by users but should be based on these core types.

---

## Database Schema

### Group and User Relationships

```sql
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    hidden BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE grouprel (
    user_id UUID NOT NULL,
    group_id UUID NOT NULL,
    PRIMARY KEY (user_id, group_id)
);
```

```sql
CREATE TABLE group_closure (
    ancestor_id UUID NOT NULL,
    descendant_id UUID NOT NULL,
    depth INTEGER NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id)
);
```

This schema allows efficient queries of all descendants or ancestors of any group. While this requires more initial upkeep to define each relationship it makes the system more resistant to denial of service attacks oriented around forcing recursive calls.

## Entity Sharing

An entity has a group_id for the group that owns it, but also can have multiple groups that it is in that have access to it.
```sql
CREATE TABLE entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id uuid references groups(id),
    -- additional fields ...
);

CREATE TABLE entity_groups (
    entity_id UUID NOT NULL,
    group_id UUID NOT NULL,
    PRIMARY KEY (entity_id, group_id)
);
```

