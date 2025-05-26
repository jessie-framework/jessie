///AppInfo is a struct that jessie-lib uses to define platform specific behaviour.
///You can edit it using config.ron at the root of your project.
pub struct AppInfo {
    linux: LinuxConfig,
    windows: WindowsConfig,
    macos: MacOSConfig,
    android: AndroidConfig,
    ios: IOSConfig,
}

///A struct for Linux specific configuration.
pub struct LinuxConfig {}

///A struct for Windows specific configuration.
pub struct WindowsConfig {}

///A struct for MacOS specific configuration.
pub struct MacOSConfig {}

///A struct for Android specific configuration.
pub struct AndroidConfig {}

///A struct for IOS specific configuration.
pub struct IOSConfig {}
