//! Endpoints for registering, logging in, logging out, performing OAuth flows

mod attest;
mod discord;
mod login;
mod logout;
mod register;

pub use attest::*;
pub use discord::*;
pub use login::*;
pub use logout::*;
pub use register::*;
