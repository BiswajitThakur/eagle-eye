#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "sync")]
mod sync;

#[cfg(feature = "sync")]
pub use sync::EagleEyeServerSync;
