use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

use failure::Fail;

pub mod backup;
pub mod error;
pub mod restore;
pub mod source;

pub mod repository;

pub struct Version(String);

struct Index {
}
