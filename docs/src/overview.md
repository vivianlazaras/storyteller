# Storyteller Overview

## Current Structure (07/12/25)

The codebase is split into two primary components as it currently stands. These components are the API backend, and the rust middleware that handles server side template rendering.

## API Backend

The primary responsibility of the API backend within the current implementation are as follows:
1. Handle database Create, Read, Update, Delete (CRUD) soperations
2. Authenticate users.
3. Issue JSON web tokens used as authentication tokens.

## Rust Middleware

The responsibilities of the rust middleware as they currently stand are:
1. Handling form submission and santization.
2. Handle image uploads, exif tag extraction, and image disk operations.
3. Handle graphviz DOT structure rendering for existing graphs.
4. Compile and serve tera templates as part of the current web UI.
5. Grab cookies from web requests and convert into Authorization field in http headers for API authentication.
6. Handle authentication checks to ensure the user is authorized (if not backend will just return an error rather than redirecting).

## Entities

Entities is an abstraction used to refer to any first class item operated on by this project, currently that includes:
1. Stories
2. Fragments
3. Characters
4. Locations
5. Tags
6. Graphs
7. Notes

Within this model entities are linked using relationships which are primarily hierarcical by nature, a story may have locations, characters, fragments, notes, a character may have fragments, notes, images. While these aren't all the allow relationships this works to describe the key feature of this codebase, that of defining related entities.

## Graphs
Some graph generation can be done automatically, this includes timelines currently, but will also include family trees, location's event history, and general relationship maps. Given the complexity of graph algorithims this currently isn't implemented.

### Manual Graph Creation
This codebase provides a means of creating graphs manually through the use of graphviz DOT structure, these graphs can represent any types of relationships between entities within the database.

## Models, Images, and Audio Files
Assets are a means of adding alternative representations of descriptive attributes of stories in addition to text descriptions, currently image processing is implemented if not fully stabalized, while models, and audio still have some work that needs to be done. The goal is to be able to link 3D models, audio, and images to content to develop a more interactive story. 

### Asset Automation
Part of the goal of this project being extensibility, the eventual intention is to provide tools for automated construction of assets, this includes AI generated text, 3D reconstruction pipelines for model generation from static images, and video files.

Currently a 3D reconstruction pipeline is implemented, but under construction for this project to allow 3D models to be generated from videos and images uploaded to the rust frontend. 

## DataSets
while not standardized datasets are an attempt to link serialized, structured data as an asset that can be linked to story fragments, locations, and characters to develop a more complete picture of the environment of the story, this can take many forms, including graphical representations of trends over time in weather, in location history for specific individuals. and many more varieties.

## Integrations
Integrations provide a means of extending the functionality of storyteller by providing custom web assembly modules that are granted API access for specific user's, as well as specific API endpoints for integration of 3rd party tooling and infrastructure such as ChatGPT, and Gemini for content suggestions. These integrations would be an opt in system per instance.

