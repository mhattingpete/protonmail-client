#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    protonmail_client_lib::run();
}
