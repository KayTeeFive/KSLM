// KSLM – KDE Simple Layout Manager (Wayland workaround)
// Copyright (C) 2026
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use zbus::{
    connection::Builder,
    interface,
    proxy,
    Connection,
};
use futures_util::StreamExt;

// ---------------------------
// Configuration
// ---------------------------

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    layouts: Vec<String>,
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("kslm.yml")
}

// Load config, or auto-generate it from KDE layouts if missing
async fn load_config(proxy: &KdeKeyboardProxy<'_>) -> Config {
    let path = config_path();

    if !path.exists() {
        info!("Config not found, detecting layouts from KDE...");

        let layouts = match proxy.get_layouts_list().await {
            Ok(raw) => {
                let detected: Vec<String> = raw
                    .iter()
                    .take(2)
                    .map(|(layout, _, _)| layout.clone())
                    .collect();

                if detected.is_empty() {
                    warn!("No layouts detected from KDE, config will be empty");
                }

                detected
            }
            Err(e) => {
                error!("Failed to get layouts from KDE: {}", e);
                vec![]
            }
        };

        let cfg = Config { layouts };

        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match serde_yaml::to_string(&cfg) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, &content) {
                    error!("Failed to write config: {}", e);
                } else {
                    info!("Config created: {:?}", path);
                }
            }
            Err(e) => error!("Failed to serialize config: {}", e),
        }

        return cfg;
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_yaml::from_str(&content).unwrap_or_else(|e| {
            error!("Config parse error: {}", e);
            Config { layouts: vec![] }
        }),
        Err(e) => {
            error!("Config read error: {}", e);
            Config { layouts: vec![] }
        }
    }
}

// ---------------------------
// KDE Keyboard D-Bus proxy
// ---------------------------

#[proxy(
    interface = "org.kde.KeyboardLayouts",
    default_service = "org.kde.keyboard",
    default_path = "/Layouts"
)]
trait KdeKeyboard {
    #[zbus(name = "getLayoutsList")]
    fn get_layouts_list(&self) -> zbus::Result<Vec<(String, String, String)>>;

    #[zbus(name = "getLayout")]
    fn get_layout(&self) -> zbus::Result<u32>;

    #[zbus(name = "setLayout")]
    fn set_layout(&self, index: u32) -> zbus::Result<bool>;

    #[zbus(signal, name = "layoutListChanged")]
    fn layout_list_changed(&self) -> zbus::Result<()>;

    #[zbus(signal, name = "layoutChanged")]
    fn layout_changed(&self, index: u32) -> zbus::Result<()>;
}

// ---------------------------
// Layout state
// ---------------------------

#[derive(Debug, Clone, PartialEq)]
struct LayoutEntry {
    index: u32,
    layout: String,
    #[allow(dead_code)]
    variant: String,
    name: String,
}

#[derive(Debug, Default)]
struct LayoutState {
    layouts: Vec<LayoutEntry>,
    layout_map: HashMap<String, u32>, // layout code -> index
}

impl LayoutState {
    fn update(&mut self, raw: Vec<(String, String, String)>) -> bool {
        let new_layouts: Vec<LayoutEntry> = raw
            .into_iter()
            .enumerate()
            .map(|(i, (layout, variant, name))| LayoutEntry {
                index: i as u32,
                layout,
                variant,
                name,
            })
            .collect();

        if new_layouts == self.layouts {
            return false;
        }

        self.layouts = new_layouts;
        self.layout_map = self
            .layouts
            .iter()
            .map(|l| (l.layout.clone(), l.index))
            .collect();

        info!("Layout map updated:");
        for l in &self.layouts {
            info!("{:2}  {:3}  {}", l.index, l.layout, l.name);
        }

        true
    }

    fn layout_code_by_index(&self, index: u32) -> Option<&str> {
        self.layouts
            .get(index as usize)
            .map(|l| l.layout.as_str())
    }
}

// ---------------------------
// Shared daemon state
// ---------------------------

type SharedState = Arc<RwLock<LayoutState>>;

async fn refresh_layouts(proxy: &KdeKeyboardProxy<'_>, state: &SharedState) {
    match proxy.get_layouts_list().await {
        Ok(raw) => {
            state.write().await.update(raw);
        }
        Err(e) => error!("Failed to refresh layouts: {}", e),
    }
}

async fn get_current_layout(proxy: &KdeKeyboardProxy<'_>, state: &SharedState) -> Option<String> {
    match proxy.get_layout().await {
        Ok(index) => {
            let st = state.read().await;
            st.layout_code_by_index(index).map(|s| s.to_string())
        }
        Err(e) => {
            error!("Failed to get current layout: {}", e);
            None
        }
    }
}

async fn next_layout(
    proxy: &KdeKeyboardProxy<'_>,
    state: &SharedState,
    cfg_layouts: &[String],
) {
    let current = get_current_layout(proxy, state).await;
    let current = current.as_deref().unwrap_or("");

    let target = if let Some(pos) = cfg_layouts.iter().position(|l| l == current) {
        &cfg_layouts[(pos + 1) % cfg_layouts.len()]
    } else {
        &cfg_layouts[0]
    };

    let target_index = {
        let st = state.read().await;
        st.layout_map.get(target.as_str()).copied()
    };

    match target_index {
        Some(idx) => match proxy.set_layout(idx).await {
            Ok(_) => info!("Switched layout to: {}", target),
            Err(e) => error!("Failed to set layout: {}", e),
        },
        None => error!("Layout '{}' not found in KDE layout list", target),
    }
}

// ---------------------------
// D-Bus service object
// ---------------------------

struct KslmService {
    proxy: Arc<KdeKeyboardProxy<'static>>,
    state: SharedState,
    cfg_layouts: Vec<String>,
}

#[interface(name = "org.kslm.LayoutDaemon")]
impl KslmService {
    #[zbus(name = "toggle")]
    async fn toggle(&self) {
        next_layout(&self.proxy, &self.state, &self.cfg_layouts).await;
    }
}

// ---------------------------
// Signal listeners
// ---------------------------

async fn listen_signals(
    proxy: Arc<KdeKeyboardProxy<'static>>,
    state: SharedState,
) {
    let proxy_list = proxy.clone();
    let proxy_change = proxy.clone();
    let state_list = state.clone();
    let state_change = state.clone();

    // layoutListChanged
    tokio::spawn(async move {
        match proxy_list.receive_layout_list_changed().await {
            Ok(mut stream) => {
                while let Some(_signal) = stream.next().await {
                    debug!("Layout list changed signal received!");
                    refresh_layouts(&proxy_list, &state_list).await;
                }
            }
            Err(e) => error!("Failed to subscribe to layoutListChanged: {}", e),
        }
    });

    // layoutChanged
    tokio::spawn(async move {
        match proxy_change.receive_layout_changed().await {
            Ok(mut stream) => {
                while let Some(signal) = stream.next().await {
                    if let Ok(args) = signal.args() {
                        let index = args.index();
                        let st = state_change.read().await;
                        match st.layout_code_by_index(*index) {
                            Some(code) => debug!("Layout changed signal: {}", code),
                            None => debug!("Layout changed signal: {}", index),
                        }
                    }
                }
            }
            Err(e) => error!("Failed to subscribe to layoutChanged: {}", e),
        }
    });
}

// ---------------------------
// Entry point
// ---------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kslm=info".parse().unwrap()),
        )
        .init();

    // Connect to session D-Bus
    let conn = Connection::session().await?;

    // Create KDE keyboard proxy
    let proxy = Arc::new(KdeKeyboardProxy::new(&conn).await?);

    let state: SharedState = Arc::new(RwLock::new(LayoutState::default()));

    // Initial layout refresh — must happen before load_config
    // so the state is populated and config can read layout names
    refresh_layouts(&proxy, &state).await;

    // Load config — auto-generates from KDE layouts if missing
    let cfg = load_config(&proxy).await;
    info!("Active config layouts: {:?}", cfg.layouts);

    // Start signal listeners
    listen_signals(proxy.clone(), state.clone()).await;

    // Register our D-Bus service
    let service = KslmService {
        proxy: proxy.clone(),
        state: state.clone(),
        cfg_layouts: cfg.layouts.clone(),
    };

    let _conn = Builder::session()?
        .name("org.kslm.LayoutDaemon")?
        .serve_at("/org/kslm/LayoutDaemon", service)?
        .build()
        .await?;

    info!("Layout daemon running...");

    // Keep running forever
    std::future::pending::<()>().await;
    Ok(())
}