use std::fmt;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Error, Serialize, Deserialize, Debug)]
pub enum ServerError {
    Configuration(String),
    RepoInit(String)
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
