# StoryTeller
This project is a tool for creating stories for tabletop, fiction writing, non fiction, video game design or any other kind of story.

## Entity Model
Each entity represents a fundamental of story creation
1. Who - Characters
2. What - Fragments
3. Where - Locations
4. When - Timelines
5. How - Methods
6. Why - Motivations
7. Patterns - Patterns (recurring event types)
8. Interpretation - Interpretation
How the user recieves and reinterprets the story 

## Database backends
1. Postgresql
currently this code has only been tested with postgresql but in theory
any database that is supported by gorm should work.

## Future Plans
1. Complete token exchange allowing the API to be exposed to the WAN for homeserver federation.
2. import timelines from immich.
3. Create a wasmtime based plugin system.
4. Use OpenPose + OpenMVG or similar software to generate 3D models from images of people.
5. Implement a format for providing datasets to enrich stories, such as climate change data, map data, etc to provide custom visual components.
6. Explicit Group Management.
7. Image Albums.
8. Character Groups / Organizations.
9. Ability to Merge fragments.
10. Standardize graph representation to allow user's to use dot format to define custom graphs.

## Would Be Cool One Day
1. Have an AI generate tag suggestions based on story content.

## Just A Thought:
Once something is published, anyone can read, and modify it, for the longest time we've tried to create intellectual property laws to protect the quality of original work by puishing "theft" but what if we reimagined how we approach artistic works, after all as soon as something is read, often even by the author it carries a slightly different meaning than when it was written, so what "theft" are we punishing? 

I propse instead that we encourage, and provide means for people to expand upon any idea published that exists within the public domain, when a user requests a story they shall see only what the creator/creators have produced, but anyone can edit a story and those edits are made visible as "other peoples interpretation / take" on the story, allowing the story to take on a life of its own through how people interact with it, while still preserving the intention of the original creator/creators. Now to implement this lol.