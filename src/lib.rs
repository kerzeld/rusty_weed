#[macro_use(concat_string)]
extern crate concat_string;

/// Contains the [master](crate::master::Master) struct that implements all master server endpoints
pub mod master;

/// Contains the [volume](crate::volume::Volume) struct that implements all volume server endpoints
pub mod volume;

/// Holds universal structs like the [FID](crate::utils::FID) and [Locations](crate::utils::Location)
pub mod utils;
