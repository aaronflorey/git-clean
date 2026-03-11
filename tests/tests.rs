#![deny(warnings)]

// Module full of support functions and structs for integration tests
mod support;

// Actual integration tests
mod deletion;
mod local;
mod remote;
mod utility;
