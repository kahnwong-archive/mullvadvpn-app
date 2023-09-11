use anyhow::{anyhow, Result};
use mullvad_management_interface::MullvadProxyClient;
use mullvad_types::api_access_method::{AccessMethod, ApiAccessMethodReplace};
use std::net::IpAddr;

use clap::{Args, Subcommand};
use talpid_types::net::openvpn::SHADOWSOCKS_CIPHERS;

#[derive(Subcommand, Debug)]
pub enum Proxy {
    /// Get current api settings
    #[clap(subcommand)]
    Api(ApiCommands),
}

impl Proxy {
    pub async fn handle(self) -> Result<()> {
        match self {
            Proxy::Api(cmd) => match cmd {
                ApiCommands::List => {
                    //println!("Listing the API access methods: ..");
                    Self::list().await?;
                }
                ApiCommands::Add(cmd) => {
                    //println!("Adding custom proxy");
                    Self::add(cmd).await?;
                }
                ApiCommands::Edit(cmd) => {
                    // Transform human-readable index to 0-based indexing.
                    let index = Self::zero_to_one_based_index(cmd.index)?;
                    Self::edit(EditCustomCommands { index, ..cmd }).await?
                }
                ApiCommands::Remove(cmd) => {
                    // Transform human-readable index to 0-based indexing.
                    let index = Self::zero_to_one_based_index(cmd.index)?;
                    Self::remove(RemoveCustomCommands { index }).await?
                }
            },
        };
        Ok(())
    }

    /// Show all API access methods.
    async fn list() -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        for (index, api_access_method) in rpc.get_api_access_methods().await?.iter().enumerate() {
            println!("{}. {:?}", index + 1, api_access_method);
        }
        Ok(())
    }

    /// Add a custom API access method.
    async fn add(cmd: AddCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let proxy = AccessMethod::try_from(cmd.clone())?;
        rpc.add_access_method(proxy).await?;
        Ok(())
    }

    /// Remove an API access method.
    async fn remove(cmd: RemoveCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        let access_method = rpc
            .get_api_access_methods()
            .await?
            .get(cmd.index)
            .ok_or(anyhow!(format!(
                "Access method {} does not exist",
                cmd.index + 1
            )))?
            .clone();
        rpc.remove_access_method(access_method)
            .await
            .map_err(Into::<anyhow::Error>::into)
    }

    async fn edit(cmd: EditCustomCommands) -> Result<()> {
        let mut rpc = MullvadProxyClient::new().await?;
        // Retrieve the access method to edit
        let access_method = rpc
            .get_api_access_methods()
            .await?
            .get(cmd.index)
            .ok_or(anyhow!(format!(
                "Access method {} does not exist",
                cmd.index + 1
            )))?
            .clone();

        // Create a new access method combining the new params with the previous values
        let edited_access_method: AccessMethod = match access_method {
            AccessMethod::Shadowsocks(shadowsocks) => {
                let ip = cmd.params.ip.unwrap_or(shadowsocks.peer.ip()).to_string();
                let port = cmd.params.port.unwrap_or(shadowsocks.peer.port());
                let password = cmd.params.password.unwrap_or(shadowsocks.password);
                let cipher = cmd.params.cipher.unwrap_or(shadowsocks.cipher);
                mullvad_types::api_access_method::Shadowsocks::from_args(ip, port, cipher, password)
                    .map(|x| x.into())
            }
            AccessMethod::Socks5(socks) => match socks {
                mullvad_types::api_access_method::Socks5::Local(local) => {
                    let ip = cmd.params.ip.unwrap_or(local.peer.ip()).to_string();
                    let port = cmd.params.port.unwrap_or(local.peer.port());
                    let local_port = cmd.params.local_port.unwrap_or(local.port);
                    mullvad_types::api_access_method::Socks5Local::from_args(ip, port, local_port)
                        .map(|x| x.into())
                }
                mullvad_types::api_access_method::Socks5::Remote(remote) => {
                    let ip = cmd.params.ip.unwrap_or(remote.peer.ip()).to_string();
                    let port = cmd.params.port.unwrap_or(remote.peer.port());
                    mullvad_types::api_access_method::Socks5Remote::from_args(ip, port)
                        .map(|x| x.into())
                }
            },
        }
        .ok_or(anyhow!(
            "Could not edit access method {}, reverting changes.",
            cmd.index
        ))?;

        rpc.replace_access_method(ApiAccessMethodReplace {
            index: cmd.index,
            access_method: edited_access_method,
        })
        .await?;

        Ok(())
    }

    fn zero_to_one_based_index(index: usize) -> Result<usize> {
        index
            .checked_sub(1)
            .ok_or(anyhow!("Access method 0 does not exist"))
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum ApiCommands {
    /// List the configured API proxies
    List,
    /// Add a custom API proxy
    #[clap(subcommand)]
    Add(AddCustomCommands),
    /// Edit an API proxy
    Edit(EditCustomCommands),
    /// Remove an API proxy
    Remove(RemoveCustomCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum AddCustomCommands {
    /// Configure SOCKS5 proxy
    #[clap(subcommand)]
    Socks5(Socks5AddCommands),

    /// Configure bundled Shadowsocks proxy
    Shadowsocks {
        /// The IP of the remote Shadowsocks server
        remote_ip: IpAddr,
        /// The port of the remote Shadowsocks server
        #[arg(default_value = "443")]
        remote_port: u16,
        /// Password for authentication
        #[arg(default_value = "mullvad")]
        password: String,
        /// Cipher to use
        #[arg(value_parser = SHADOWSOCKS_CIPHERS, default_value = "aes-256-gcm")]
        cipher: String,
    },
}

#[derive(Args, Debug, Clone)]
pub struct EditCustomCommands {
    /// Which API proxy to edit
    index: usize,
    /// Editing parameters
    #[clap(flatten)]
    params: EditParams,
}

#[derive(Args, Debug, Clone)]
pub struct RemoveCustomCommands {
    /// Which API proxy to remove
    index: usize,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Socks5AddCommands {
    /// Configure a local SOCKS5 proxy
    Local {
        /// The port that the server on localhost is listening on
        local_port: u16,
        /// The IP of the remote peer
        remote_ip: IpAddr,
        /// The port of the remote peer
        remote_port: u16,
    },
    /// Configure a remote SOCKS5 proxy
    Remote {
        /// The IP of the remote proxy server
        remote_ip: IpAddr,
        /// The port of the remote proxy server
        remote_port: u16,
        /// Username for authentication
        #[arg(requires = "password")]
        username: Option<String>,
        /// Password for authentication
        #[arg(requires = "username")]
        password: Option<String>,
    },
}

#[derive(Args, Debug, Clone)]
pub struct EditParams {
    /// Username for authentication [Shadowsocks]
    #[arg(long)]
    username: Option<String>,
    /// Password for authentication [Shadowsocks]
    #[arg(long)]
    password: Option<String>,
    /// Cipher to use [Shadowsocks]
    #[arg(value_parser = SHADOWSOCKS_CIPHERS, long)]
    cipher: Option<String>,
    /// The IP of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    ip: Option<IpAddr>,
    /// The port of the remote proxy server [Socks5 (Local & Remote proxy), Shadowsocks]
    #[arg(long)]
    port: Option<u16>,
    /// The port that the server on localhost is listening on [Socks5 (Local proxy)]
    #[arg(long)]
    local_port: Option<u16>,
}

/// Implement conversions from CLI types to Daemon types.
///
/// Since these are not supposed to be used outside of the CLI,
/// we define them in a hidden-away module.
mod conversions {
    use anyhow::{anyhow, Error};
    use mullvad_types::api_access_method as daemon_types;

    use super::{AddCustomCommands, Socks5AddCommands};

    impl TryFrom<AddCustomCommands> for daemon_types::AccessMethod {
        type Error = Error;

        fn try_from(value: AddCustomCommands) -> Result<Self, Self::Error> {
            Ok(match value {
                AddCustomCommands::Socks5(variant) => match variant {
                    Socks5AddCommands::Local {
                        local_port,
                        remote_ip,
                        remote_port,
                    } => {
                        println!("Adding LOCAL SOCKS5-proxy: localhost:{local_port} => {remote_ip}:{remote_port}");
                        let socks_proxy = daemon_types::Socks5::Local(
                            daemon_types::Socks5Local::from_args(
                                remote_ip.to_string(),
                                remote_port,
                                local_port,
                            )
                            .ok_or(anyhow!("Could not create a local Socks5 api proxy"))?,
                        );
                        daemon_types::AccessMethod::Socks5(socks_proxy)
                    }
                    Socks5AddCommands::Remote {
                        remote_ip,
                        remote_port,
                        username,
                        password,
                    } => {
                        println!("Adding REMOTE SOCKS5-proxy: {username:?}+{password:?} @ {remote_ip}:{remote_port}");
                        let socks_proxy = daemon_types::Socks5::Remote(
                            daemon_types::Socks5Remote::from_args(
                                remote_ip.to_string(),
                                remote_port,
                            )
                            .ok_or(anyhow!("Could not create a remote Socks5 api proxy"))?,
                        );
                        daemon_types::AccessMethod::Socks5(socks_proxy)
                    }
                },
                AddCustomCommands::Shadowsocks {
                    remote_ip,
                    remote_port,
                    password,
                    cipher,
                } => {
                    println!(
                "Adding Shadowsocks-proxy: {password} @ {remote_ip}:{remote_port} using {cipher}"
                    );
                    let shadowsocks_proxy = daemon_types::Shadowsocks::from_args(
                        remote_ip.to_string(),
                        remote_port,
                        cipher,
                        password,
                    )
                    .ok_or(anyhow!("Could not create a Shadowsocks api proxy"))?;
                    daemon_types::AccessMethod::Shadowsocks(shadowsocks_proxy)
                }
            })
        }
    }
}
