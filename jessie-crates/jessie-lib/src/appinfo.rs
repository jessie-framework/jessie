use serde::{Deserialize, Serialize};

///AppInfo is a struct that jessie-lib uses to define platform specific behaviour.
///You can edit it using config.ron at the root of your project.
#[derive(Deserialize, Serialize)]
pub struct AppInfo {
    linux: LinuxConfig,
    windows: WindowsConfig,
    macos: MacOSConfig,
    android: AndroidConfig,
    ios: IOSConfig,
}

///A struct for Linux specific configuration.
#[derive(Deserialize, Serialize)]
pub struct LinuxConfig {}

///A struct for Windows specific configuration.
#[derive(Deserialize, Serialize)]
pub struct WindowsConfig {}

///A struct for MacOS specific configuration.
#[derive(Deserialize, Serialize)]
pub struct MacOSConfig {}

///A struct for Android specific configuration.
#[derive(Deserialize, Serialize)]
pub struct AndroidConfig {}

///A struct for IOS specific configuration.
#[derive(Deserialize, Serialize)]
pub struct IOSConfig {}
