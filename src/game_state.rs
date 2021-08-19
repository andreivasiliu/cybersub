//! This module represent all the physical game state, without the visual draw
//! state.
//!
//! All game state and update logic in this module is fully deterministic:
//! * All operations use integer arithmetic
//! * All operations are either well-ordered, or order-agnostic
//! * State is losslessly serializable and can be transferred over a network
//! * The only public way to modify the state is via a vector of commands
//!
//! Although not yet fully realized, the data is layed out so that the various
//! update modules can run in parallel, while still being deterministic.

pub(crate) mod collisions;
pub(crate) mod objects;
pub(crate) mod rocks;
pub(crate) mod sonar;
pub(crate) mod state;
pub(crate) mod update;
pub(crate) mod water;
pub(crate) mod wires;
