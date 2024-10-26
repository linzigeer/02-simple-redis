mod hmap;
mod map;

use std::string::FromUtf8Error;

use crate::{Backend, RespArray, RespError, RespFrame};
use lazy_static::lazy_static;
use thiserror::Error;

lazy_static! {
    static ref RESP_OK: RespFrame = RespFrame::SimpleString("OK".into());
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("invalid command:{0}")]
    InvalidCommand(String),

    #[error("invalid argument:{0}")]
    InvalidArgument(String),

    #[error("{0}")]
    RespError(#[from] RespError),

    #[error("{0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}

pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

pub enum Command {
    Set(Set),
    Get(Get),
    HSet(HSet),
    HGet(HGet),
    HGetAll(HGetAll),
}

#[derive(Debug, PartialEq)]
pub struct Set {
    pub key: String,
    pub value: RespFrame,
}

impl Set {
    fn new(key: String, value: RespFrame) -> Self {
        Self { key, value }
    }
}

#[derive(Debug, PartialEq)]
pub struct Get {
    pub key: String,
}

impl Get {
    fn new(key: String) -> Self {
        Self { key }
    }
}

#[derive(Debug)]
pub struct HSet {
    pub table_name: String,
    pub key: String,
    pub value: RespFrame,
}

impl HSet {
    pub fn new(table_name: String, key: String, value: RespFrame) -> Self {
        Self {
            table_name,
            key,
            value,
        }
    }
}

#[derive(Debug)]
pub struct HGet {
    pub table_name: String,
    pub key: String,
}

impl HGet {
    pub fn new(table_name: String, key: String) -> Self {
        Self { table_name, key }
    }
}

#[derive(Debug)]
pub struct HGetAll {
    pub table_name: String,
}

impl HGetAll {
    pub fn new(table_name: String) -> Self {
        Self { table_name }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;

    fn try_from(_: RespArray) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub fn validate_command(
    value: &RespArray,
    command_name: &'static str,
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() != 1 + n_args {
        return Err(CommandError::InvalidArgument(format!(
            "{command_name} command should have exactly {n_args} argument(s)!",
        )));
    }

    match value[0] {
        RespFrame::BulkString(ref cmd) => {
            if cmd.as_ref().to_ascii_lowercase() != command_name.as_bytes() {
                return Err(CommandError::InvalidCommand(format!(
                    "expect {} got {}",
                    command_name,
                    String::from_utf8_lossy(cmd.as_ref())
                )));
            }
        }

        _ => {
            return Err(CommandError::InvalidCommand(
                "cmd expect to be BulkString type!".into(),
            ));
        }
    };

    Ok(())
}

pub fn extract_cmd_args(
    value: RespArray,
    skip_index: usize,
) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(skip_index).collect())
}
