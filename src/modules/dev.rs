//! Developer tools: a parent with no functions of its own.

pub mod binary;
pub mod colors;
pub mod net;

use crate::modules::Module;

pub const MODULE: Module = Module {
    name: "dev",
    about: "developer tools: IP addresses and subnets, raw bytes, colors",
    children: &[net::MODULE, binary::MODULE, colors::MODULE],
    ..Module::EMPTY
};
