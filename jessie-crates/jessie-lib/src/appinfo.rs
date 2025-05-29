use serde::{Deserialize, Serialize};

///AppInfo is a struct that jessie-lib uses to define platform specific behaviour.
///You can edit it using config.ron at the root of your project.
#[derive(Deserialize, Serialize, Debug)]
pub struct AppInfo {
    linux: Option<LinuxConfig>,
    windows: Option<WindowsConfig>,
    macos: Option<MacOSConfig>,
    android: Option<AndroidConfig>,
    ios: Option<IOSConfig>,
}

///A struct for Linux specific configuration.
#[derive(Deserialize, Serialize, Debug)]
pub struct LinuxConfig {}

///A struct for Windows specific configuration.
#[derive(Deserialize, Serialize, Debug)]
pub struct WindowsConfig {}

///A struct for MacOS specific configuration.
#[derive(Deserialize, Serialize, Debug)]
pub struct MacOSConfig {}

///A struct for Android specific configuration.
#[derive(Deserialize, Serialize, Debug)]
pub struct AndroidConfig {}

///A struct for IOS specific configuration.
#[derive(Deserialize, Serialize, Debug)]
pub struct IOSConfig {}
