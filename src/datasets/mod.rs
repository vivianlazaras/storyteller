//! This module provides routes, and functions for operating with datasets relavent to stories.
//!
//! I want to provide a means of integrating custom datasets to provide a more rich landscape for using integrations
//! for example one potential data source might be climate data for a given location.

use rocket::{Route, routes, get, post, FromForm};


pub fn get_routes() -> Vec<Route> {
    routes![]
}