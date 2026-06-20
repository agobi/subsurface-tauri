// AI-generated (Claude)
// Ambient declarations for @tauri-apps/plugin-os (not yet in npm dependencies).
declare module "@tauri-apps/plugin-os" {
  export type Platform =
    | "linux"
    | "macos"
    | "ios"
    | "freebsd"
    | "dragonfly"
    | "netbsd"
    | "openbsd"
    | "solaris"
    | "android"
    | "windows";

  export function platform(): Promise<Platform>;
}
