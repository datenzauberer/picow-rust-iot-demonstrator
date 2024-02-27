//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;

use if_addrs::get_if_addrs;
use if_addrs::IfAddr;

fn main() {
    load_dotenv_from_parent_directory();

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    set_data_bridge_host_ip_and_port();
}

fn load_dotenv_from_parent_directory() {
    // Get the current directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    // Construct the path for one level up
    let parent_dir = current_dir
        .parent()
        .expect("Failed to find parent directory");
    // Append the .env filename to the parent directory path
    let dotenv_path = parent_dir.join(".env");
    dotenvy::from_path(dotenv_path).expect("Failed to load .env file");
}

// Configuration for the Data Bridge Endpoint
// (to which the picow sends data)
// Following workflow:
// ../.env -1-> env variables -2-> main.rs
// 1) In the build process the env variable IOT_DATA_BRIDGE_URL is extracted
//    If no host is configured, on linux the first wifi adapter is used,
//    on other oses an error is thrown
// 2) Env variables are set (and inserted in the main.rs)
//    IOT_DATA_BRIDGE_HOST_IP, IOT_DATA_BRIDGE_PORT

fn set_data_bridge_host_ip_and_port() {
    let serverurl =
        env::var("IOT_DATA_BRIDGE_URL").expect("env variable IOT_DATA_BRIDGE_URL not set");
    let s = serverurl.split(':').collect::<Vec<&str>>();
    let (ip, port) = if serverurl.starts_with(':') {
        if !cfg!(target_os = "linux") {
            panic!(
                r"Automatic detection of wlan interfaces only works on linux.
Please configure IOT_DATA_BRIDGE_URL in .env with the IP Address (before the port).
You can get the ipaddress:
Windows: ipconfig
Mac Os: ifconfig en0
"
            );
        }
        let ip = get_ipv4_of_first_wifi_adapter().unwrap_or_else(|err| panic!("{}", err));
        (ip.to_string(), s[1].to_string())
    } else {
        if s.len() != 2 {
            panic!("IOT_DATA_BRIDGE_URL must be in form IP:PORT, e.g. 192.168.0.1:8080");
        };
        (s[0].to_string(), s[1].to_string())
    };
    println!("cargo:rustc-env=IOT_DATA_BRIDGE_HOST_IP={}", ip);
    println!("cargo:rustc-env=IOT_DATA_BRIDGE_PORT={}", port);
}

fn get_ipv4_of_first_wifi_adapter() -> Result<IpAddr, String> {
    let wifi_interfaces = get_wifi_interfaces()?;
    let addrs = get_if_addrs().map_err(|_| "Failed to get network interfaces".to_string())?;
    let wifi_adapters: Vec<_> = addrs
        .into_iter()
        .filter(|addr| {
            // Ensure the address is not a loopback
            !addr.is_loopback() &&
            // Ensure the address is IPv4
            matches!(addr.addr, IfAddr::V4(_)) &&
            // Check if the interface name is in the list of WiFi interfaces
            wifi_interfaces.contains(&addr.name)
        })
        .collect();

    match wifi_adapters.len() {
        0 => Err("No Wi-Fi adapter found".to_string()),
        1 => Ok(wifi_adapters[0].addr.ip()),
        _ => {
            let err_msg = wifi_adapters
                .iter()
                .map(|adapter| format!("{}: {}", adapter.name, adapter.addr.ip()))
                .collect::<Vec<_>>()
                .join("\n");
            Err(format!("Multiple Wi-Fi adapters found:\n{}", err_msg))
        }
    }
}

fn get_wifi_interfaces() -> Result<Vec<String>, String> {
    // Path to the wireless network information
    let path = "/proc/net/wireless";

    // Attempt to read the file content, returning an error message if failed
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read /proc/net/wireless: {}", e)),
    };

    // Extracting interface names
    let interfaces: Vec<String> = content
        .lines()
        .skip(2) // Skip the header lines
        .filter_map(|line| {
            line.split_whitespace()
                .next()
                // Remove a trailing colon from the interface name, if present
                .map(|name| name.trim_end_matches(':').to_string())
        })
        .collect();

    Ok(interfaces)
}
