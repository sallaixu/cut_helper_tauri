pub mod cut_admin;
pub mod image_processor;

#[cfg(target_os = "windows")]
pub mod keyboard_sim;
#[cfg(target_os = "windows")]
pub mod model_manager;
#[cfg(target_os = "windows")]
pub mod speech;
