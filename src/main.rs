use anyhow::anyhow;
use embedded_svc::{httpd::registry::*, httpd::*, wifi::*};
use esp_idf_svc::{httpd as idfhttpd, netif::*, nvs::EspDefaultNvs, sysloop::*, wifi::*};
use log::*;
use std::{sync::Arc, thread, time::*};

mod webconfig;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let _wifi = wifi(netif_stack, sys_loop_stack, default_nvs)?;
    let _httpd = httpd();

    loop {
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    let ap_infos = wifi.scan()?;
    let ours = ap_infos.into_iter().find(|a| a.ssid == webconfig::SSID);

    let channel = if let Some(ours) = ours {
        Some(ours.channel)
    } else {
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: webconfig::SSID.into(),
            password: webconfig::PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");
    } else {
        return Err(anyhow!("Unexpected wifi status: {:?}", status));
    }

    Ok(wifi)
}

fn httpd() -> Result<idfhttpd::Server> {
    let server = idfhttpd::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Yeehaa!".into()))?
        .at("/kukkuu")
        .get(|_| {
            Response::new(403)
                .status_message("No permissions")
                .body("You have no permissions to access this page".into())
                .into()
        })?;

    server.start(&Default::default())
}
