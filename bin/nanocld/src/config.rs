use nanocl_stubs::config::{DaemonConfig, DaemonConfigFile};

use nanocl_utils::unix;
use nanocl_error::io::{IoResult, FromIo};

use crate::cli::Cli;

/// Merge cli and config file together to generate the daemon config
fn gen_daemon_conf(
  args: &Cli,
  config: &DaemonConfigFile,
) -> IoResult<DaemonConfig> {
  let hosts = if let Some(ref hosts) = args.hosts {
    hosts.to_owned()
  } else if let Some(ref hosts) = config.hosts {
    hosts.to_owned()
  } else {
    vec![String::from("unix:///run/nanocl/nanocl.sock")]
  };
  let state_dir = if let Some(ref state_dir) = args.state_dir {
    state_dir.to_owned()
  } else if let Some(ref state_dir) = config.state_dir {
    state_dir.to_owned()
  } else {
    String::from("/var/lib/nanocl")
  };
  let docker_host = if let Some(ref docker_host) = args.docker_host {
    docker_host.to_owned()
  } else if let Some(ref docker_host) = config.docker_host {
    docker_host.to_owned()
  } else {
    String::from("/var/run/docker.sock")
  };
  let gateway = if let Some(ref gateway) = args.gateway {
    gateway.to_owned()
  } else if let Some(ref gateway) = config.gateway {
    gateway.to_owned()
  } else {
    unix::network::get_default_ip()
      .map_err(|err| err.map_err_context(|| "Gateway"))?
      .to_string()
  };
  let hostname = if let Some(ref hostname) = args.hostname {
    hostname.to_owned()
  } else if let Some(ref hostname) = config.hostname {
    hostname.to_owned()
  } else {
    unix::network::get_hostname()
      .map_err(|err| err.map_err_context(|| "Hostname"))?
  };
  let advertise_addr = if let Some(ref advertise_addr) = args.advertise_addr {
    advertise_addr.to_owned()
  } else {
    gateway.clone()
  };
  let store_addr = if let Some(ref store_addr) = args.store_addr {
    Some(store_addr.to_owned())
  } else {
    config.store_addr.clone()
  };
  Ok(DaemonConfig {
    hosts,
    gateway,
    hostname,
    state_dir,
    docker_host,
    store_addr,
    gid: args.gid,
    advertise_addr,
    nodes: args.nodes.clone(),
    conf_dir: args.conf_dir.clone(),
    ssl: args.ssl.clone(),
  })
}

/// Read config file from config_dir
fn read_config_file(config_dir: &str) -> IoResult<DaemonConfigFile> {
  let config_path = std::path::Path::new(&config_dir).join("nanocl.conf");
  if !config_path.exists() {
    return Ok(DaemonConfigFile::default());
  }
  let content = std::fs::read_to_string(&config_path).map_err(|err| {
    err.map_err_context(|| {
      format!(
        "Error while reading config file at {}",
        config_path.display()
      )
    })
  })?;
  let config = serde_yaml::from_str::<DaemonConfigFile>(&content)
    .map_err(|err| err.map_err_context(|| "DaemonConfigFile"))?;
  Ok(config)
}

/// Init Daemon config
/// It will read /etc/nanocl/nanocl.conf
/// and parse Cli arguments we merge them together with a priority to Cli arguments
pub fn init(args: &Cli) -> IoResult<DaemonConfig> {
  let file_config = read_config_file(&args.conf_dir)?;
  gen_daemon_conf(args, &file_config)
}

/// Config unit test
#[cfg(test)]
mod tests {
  use std::os::unix::prelude::PermissionsExt;

  use super::*;

  /// Test merge config
  #[test]
  fn merge_config() {
    let args = Cli {
      hosts: Some(vec![String::from("unix:///run/nanocl/nanocl.sock")]),
      state_dir: Some(String::from("/var/lib/nanocl")),
      docker_host: Some(String::from("/var/run/docker.sock")),
      conf_dir: String::from("/etc/nanocl"),
      ..Default::default()
    };
    let config = DaemonConfigFile {
      hosts: Some(vec![String::from("unix:///run/nanocl/nanocl.sock")]),
      state_dir: Some(String::from("/var/lib/nanocl")),
      docker_host: Some(String::from("/var/run/docker.sock")),
      store_addr: None,
      gateway: None,
      hostname: None,
    };
    let merged = gen_daemon_conf(&args, &config).unwrap();
    assert_eq!(merged.hosts, args.hosts.unwrap());
    assert_eq!(merged.state_dir, args.state_dir.unwrap());
    assert_eq!(merged.docker_host, args.docker_host.unwrap());
  }

  /// Test read config file
  #[test]
  fn read_from_file() {
    let config_dir = String::from("/tmp");
    let config_path = std::path::Path::new(&config_dir).join("nanocl.conf");
    // Ensure the test file is removed
    if config_path.exists() {
      std::fs::remove_file(&config_path).unwrap();
    }
    // It should return a default config if the file does not exist
    let config = read_config_file(&config_dir);
    assert!(config.is_ok());
    assert_eq!(config.unwrap(), DaemonConfigFile::default());
    // It should return a config if the file exist
    let content = r#"state_dir: /var/lib/nanocl"#;
    std::fs::write(&config_path, content).unwrap();
    let config = read_config_file(&config_dir);
    assert!(config.is_ok());
    assert_eq!(
      config.unwrap(),
      DaemonConfigFile {
        state_dir: Some(String::from("/var/lib/nanocl")),
        ..Default::default()
      }
    );
    // It should return an error if the file is not a valid yaml
    let content = "state_dir; /var/lib/nanocl\n";
    std::fs::write(&config_path, content).unwrap();
    let config = read_config_file(&config_dir);
    assert!(config.is_err());
    // It should return an error if the file is not readable
    std::fs::set_permissions(
      &config_path,
      std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    let config = read_config_file(&config_dir);
    assert!(config.is_err());
    // It should return an error if the file is not a file
    std::fs::remove_file(&config_path).unwrap();
    std::fs::create_dir(&config_path).unwrap();
    let config = read_config_file(&config_dir);
    assert!(config.is_err());
    std::fs::remove_dir_all(&config_path).unwrap();
  }

  /// Test init config
  #[test]
  fn init_config() {
    let args = Cli {
      hosts: Some(vec![String::from("unix:///run/nanocl/nanocl.sock")]),
      state_dir: Some(String::from("/var/lib/nanocl")),
      docker_host: Some(String::from("/var/run/docker.sock")),
      conf_dir: String::from("/etc/nanocl"),
      ..Default::default()
    };
    let config = init(&args).unwrap();
    assert_eq!(config.hosts, args.hosts.unwrap());
    assert_eq!(config.state_dir, args.state_dir.unwrap());
    assert_eq!(config.docker_host, args.docker_host.unwrap());
  }
}
