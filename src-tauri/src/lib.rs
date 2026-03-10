use bollard::Docker;
use bollard::query_parameters::{
    ListContainersOptions, StartContainerOptions, RemoveContainerOptions,
    ListImagesOptions, ListVolumesOptions, RemoveImageOptions, CreateImageOptions,
    LogsOptionsBuilder,
    ListNetworksOptionsBuilder, InspectNetworkOptionsBuilder,
    RemoveVolumeOptionsBuilder, PruneVolumesOptionsBuilder,
};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::NetworkCreateRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use dashmap::DashMap;
use tauri::{State, Manager};

// ── Configuration ────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DockerConfig {
    pub docker_host: String,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            docker_host: "unix:///var/run/docker.sock".to_string(),
        }
    }
}

pub struct AppState {
    pub config: Mutex<DockerConfig>,
    pub config_path: std::path::PathBuf,
    pub terminal_sessions: Arc<DashMap<String, mpsc::Sender<Vec<u8>>>>,
}

impl AppState {
    fn load(path: std::path::PathBuf) -> Self {
        let config = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            DockerConfig::default()
        };
        Self {
            config: Mutex::new(config),
            config_path: path,
            terminal_sessions: Arc::new(DashMap::new()),
        }
    }

    fn save(&self) -> Result<(), String> {
        let config = self.config.lock().unwrap();
        let json = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }
}

fn get_docker(state: State<AppState>) -> Result<Docker, String> {
    let host = state.config.lock().unwrap().docker_host.clone();
    if host.is_empty() {
        return Docker::connect_with_local_defaults().map_err(|e: bollard::errors::Error| e.to_string());
    }
    
    if host.starts_with("unix://") {
        #[cfg(unix)]
        {
            Docker::connect_with_unix(&host[7..], 120, bollard::API_DEFAULT_VERSION)
                .map_err(|e: bollard::errors::Error| e.to_string())
        }
        #[cfg(not(unix))]
        {
            Err("Unix sockets are not supported on this platform".to_string())
        }
    } else if host.starts_with("npipe://") {
        #[cfg(target_os = "windows")]
        {
            Docker::connect_with_named_pipe(&host[8..], 120, bollard::API_DEFAULT_VERSION)
                .map_err(|e: bollard::errors::Error| e.to_string())
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err("Named pipes are only supported on Windows".to_string())
        }
    } else if host.starts_with("http://") || host.starts_with("https://") || host.starts_with("tcp://") {
        let url = if host.starts_with("tcp://") {
            host.replace("tcp://", "http://")
        } else {
            host
        };
        Docker::connect_with_http(&url, 120, bollard::API_DEFAULT_VERSION)
            .map_err(|e: bollard::errors::Error| e.to_string())
    } else {
        // Assume it might be just a path for unix socket or named pipe
        #[cfg(unix)]
        {
            if host.starts_with('/') {
                Docker::connect_with_unix(&host, 120, bollard::API_DEFAULT_VERSION)
                    .map_err(|e: bollard::errors::Error| e.to_string())
            } else {
                Docker::connect_with_local_defaults().map_err(|e: bollard::errors::Error| e.to_string())
            }
        }
        #[cfg(not(unix))]
        {
            Docker::connect_with_local_defaults().map_err(|e: bollard::errors::Error| e.to_string())
        }
    }
}

#[tauri::command]
async fn get_docker_config(state: State<'_, AppState>) -> Result<DockerConfig, String> {
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
async fn update_docker_config(state: State<'_, AppState>, config: DockerConfig) -> Result<(), String> {
    let old_host = {
        let current = state.config.lock().unwrap();
        current.docker_host.clone()
    };
    
    {
        let mut current = state.config.lock().unwrap();
        current.docker_host = config.docker_host.clone();
    }
    
    match get_docker(state.clone()) {
        Ok(docker) => {
            if let Err(e) = docker.info().await {
                let mut current = state.config.lock().unwrap();
                current.docker_host = old_host;
                return Err(format!("Failed to connect to Docker with new settings: {}", e));
            }
            // If connection works, save to file
            state.save()?;
            Ok(())
        }
        Err(e) => {
            let mut current = state.config.lock().unwrap();
            current.docker_host = old_host;
            Err(e)
        }
    }
}

// ── Docker Info ──────────────────────────────────────────

#[tauri::command]
async fn get_docker_info(state: State<'_, AppState>) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let info = docker.info().await.map_err(|e| e.to_string())?;
    serde_json::to_value(info).map_err(|e| e.to_string())
}

// ── Containers ───────────────────────────────────────────

#[tauri::command]
async fn list_containers(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let docker = get_docker(state)?;
    let options = Some(ListContainersOptions { all: true, ..Default::default() });
    let containers = docker.list_containers(options).await.map_err(|e| e.to_string())?;
    let json = serde_json::to_value(containers).map_err(|e| e.to_string())?;
    Ok(json.as_array().unwrap_or(&vec![]).clone())
}

#[tauri::command]
async fn inspect_container(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let info = docker.inspect_container(&id, None).await.map_err(|e| e.to_string())?;
    serde_json::to_value(info).map_err(|e| e.to_string())
}

#[tauri::command]
async fn container_logs(state: State<'_, AppState>, id: String, tail: Option<String>) -> Result<String, String> {
    let docker = get_docker(state)?;
    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .tail(&tail.unwrap_or_else(|| "200".to_string()))
        .build();
    let mut stream = docker.logs(&id, Some(options));
    let mut output = String::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(log) => output.push_str(&log.to_string()),
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(output)
}

#[tauri::command]
async fn start_container(state: State<'_, AppState>, id: &str) -> Result<(), String> {
    let docker = get_docker(state)?;
    docker.start_container(id, None::<StartContainerOptions>).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_container(state: State<'_, AppState>, id: &str) -> Result<(), String> {
    let docker = get_docker(state)?;
    docker.stop_container(id, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn restart_container(state: State<'_, AppState>, id: &str) -> Result<(), String> {
    let docker = get_docker(state)?;
    docker.restart_container(id, None).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn pause_container(state: State<'_, AppState>, id: &str) -> Result<(), String> {
    let docker = get_docker(state)?;
    docker.pause_container(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn unpause_container(state: State<'_, AppState>, id: &str) -> Result<(), String> {
    let docker = get_docker(state)?;
    docker.unpause_container(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_container(state: State<'_, AppState>, id: &str) -> Result<(), String> {
    let docker = get_docker(state)?;
    let options = Some(RemoveContainerOptions { force: true, v: false, link: false });
    docker.remove_container(id, options).await.map_err(|e| e.to_string())
}

// ── Terminal ─────────────────────────────────────────────

#[tauri::command]
async fn start_terminal(app: tauri::AppHandle, state: State<'_, AppState>, container_id: String, cols: u16, rows: u16) -> Result<String, String> {
    use tauri::Emitter;
    let docker = get_docker(state.clone())?;
    
    // 1. Create Exec
    let config = CreateExecOptions {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        attach_stdin: Some(true),
        tty: Some(true),
        cmd: Some(vec!["/bin/sh"]),
        env: Some(vec!["TERM=xterm"]),
        ..Default::default()
    };
    
    let exec = docker.create_exec(&container_id, config).await.map_err(|e| e.to_string())?;
    
    // 2. Start Exec
    let start_config = StartExecOptions {
        detach: false,
        ..Default::default()
    };
    
    let res = docker.start_exec(&exec.id, Some(start_config)).await.map_err(|e| e.to_string())?;

    // 3. Handle streams
    if let StartExecResults::Attached { mut output, mut input } = res {
        // Resize initially
        let resize_opt = bollard::exec::ResizeExecOptions {
            height: rows,
            width: cols,
        };
        let _ = docker.resize_exec(&exec.id, resize_opt).await;

        let exec_id_clone = exec.id.clone();
        let emit_topic = format!("terminal-output-{}", container_id);

        // Tokio Task to continuously read output from Docker and emit to frontend
        tokio::spawn(async move {
            while let Some(msg) = output.next().await {
                match msg {
                    Ok(log_output) => {
                        let bytes = log_output.into_bytes().to_vec();
                        let _ = app.emit(&emit_topic, bytes);
                    }
                    Err(e) => {
                        eprintln!("Error reading exec output: {}", e);
                        break;
                    }
                }
            }
            // Cleanup on exit
        });

        // Set up an mpsc channel to receive input from the frontend
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
        
        // Save sender to app state
        state.terminal_sessions.insert(exec.id.clone(), tx);

        // Tokio Task to read from channel and write to docker stdin
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            while let Some(data) = rx.recv().await {
                if let Err(e) = input.write_all(&data).await {
                    eprintln!("Error writing to exec stdin: {}", e);
                    break;
                }
            }
        });

        Ok(exec_id_clone)
    } else {
        Err("Failed to attach to container exec".to_string())
    }
}

#[tauri::command]
async fn write_terminal(state: State<'_, AppState>, exec_id: String, data: Vec<u8>) -> Result<(), String> {
    if let Some(tx) = state.terminal_sessions.get(&exec_id) {
        tx.send(data).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
async fn resize_terminal(state: State<'_, AppState>, exec_id: String, cols: u16, rows: u16) -> Result<(), String> {
    let docker = get_docker(state)?;
    let resize_opt = bollard::exec::ResizeExecOptions {
        height: rows,
        width: cols,
    };
    docker.resize_exec(&exec_id, resize_opt).await.map_err(|e| e.to_string())
}

// ── Images ───────────────────────────────────────────────

#[tauri::command]
async fn list_images(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let docker = get_docker(state)?;
    let options = Some(ListImagesOptions { all: true, ..Default::default() });
    let images = docker.list_images(options).await.map_err(|e| e.to_string())?;
    let json = serde_json::to_value(images).map_err(|e| e.to_string())?;
    Ok(json.as_array().unwrap_or(&vec![]).clone())
}

#[tauri::command]
async fn inspect_image(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let info = docker.inspect_image(&id).await.map_err(|e| e.to_string())?;
    serde_json::to_value(info).map_err(|e| e.to_string())
}

#[tauri::command]
async fn pull_image(app: tauri::AppHandle, state: State<'_, AppState>, name: String) -> Result<(), String> {
    use tauri::Emitter;
    let docker = get_docker(state)?;
    let (image, tag) = if name.contains(':') {
        let parts: Vec<&str> = name.splitn(2, ':').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else {
        (name.clone(), "latest".to_string())
    };
    let options = Some(CreateImageOptions {
        from_image: Some(image),
        tag: Some(tag),
        ..Default::default()
    });
    let mut stream = docker.create_image(options, None, None);
    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(err) = &info.error_detail {
                    if let Some(msg) = &err.message {
                        return Err(msg.clone());
                    }
                }
                let mut msg = String::new();
                if let Some(id) = &info.id {
                    msg.push_str(&format!("{}: ", id));
                }
                if let Some(status) = &info.status {
                    msg.push_str(status);
                }
                if let Some(prog) = &info.progress_detail {
                    if let (Some(cur), Some(tot)) = (prog.current, prog.total) {
                        let cur_mb = cur as f64 / 1_048_576.0;
                        let tot_mb = tot as f64 / 1_048_576.0;
                        msg.push_str(&format!(" ({:.1}MB / {:.1}MB)", cur_mb, tot_mb));
                    }
                }
                if !msg.is_empty() {
                    let _ = app.emit("pull-progress", msg);
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

#[tauri::command]
async fn remove_image(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = get_docker(state)?;
    let options = Some(RemoveImageOptions { force: true, ..Default::default() });
    docker.remove_image(&id, options, None).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ── Volumes ──────────────────────────────────────────────

#[tauri::command]
async fn list_volumes(state: State<'_, AppState>) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let options = Some(ListVolumesOptions { ..Default::default() });
    let volumes = docker.list_volumes(options).await.map_err(|e| e.to_string())?;
    serde_json::to_value(volumes).map_err(|e| e.to_string())
}

#[tauri::command]
async fn inspect_volume(state: State<'_, AppState>, name: String) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let vol = docker.inspect_volume(&name).await.map_err(|e| e.to_string())?;
    serde_json::to_value(vol).map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_volume(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let docker = get_docker(state)?;
    let options = RemoveVolumeOptionsBuilder::default().force(true).build();
    docker.remove_volume(&name, Some(options)).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn prune_volumes(state: State<'_, AppState>) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let options = PruneVolumesOptionsBuilder::default().build();
    let result = docker.prune_volumes(Some(options)).await.map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

// ── Networks ─────────────────────────────────────────────

#[tauri::command]
async fn list_networks(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let docker = get_docker(state)?;
    let options = ListNetworksOptionsBuilder::default().build();
    let networks = docker.list_networks(Some(options)).await.map_err(|e| e.to_string())?;
    let json = serde_json::to_value(networks).map_err(|e| e.to_string())?;
    Ok(json.as_array().unwrap_or(&vec![]).clone())
}

#[tauri::command]
async fn inspect_network(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let options = InspectNetworkOptionsBuilder::default().verbose(true).build();
    let net = docker.inspect_network(&id, Some(options)).await.map_err(|e| e.to_string())?;
    serde_json::to_value(net).map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_network(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let docker = get_docker(state)?;
    docker.remove_network(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_network(state: State<'_, AppState>, name: String, driver: Option<String>) -> Result<Value, String> {
    let docker = get_docker(state)?;
    let config = NetworkCreateRequest {
        name,
        driver: Some(driver.unwrap_or_else(|| "bridge".to_string())),
        ..Default::default()
    };
    let result = docker.create_network(config).await.map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_compose(app: tauri::AppHandle, state: State<'_, AppState>, yaml: String, project_name: String) -> Result<String, String> {
    use std::io::Write;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::process::Stdio;
    use tauri::Emitter;

    let tmp_dir = std::env::temp_dir().join("edocker_templates");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let file_path = tmp_dir.join(format!("{}.yml", project_name));
    let mut file = std::fs::File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(yaml.as_bytes()).map_err(|e| e.to_string())?;

    let host = state.config.lock().unwrap().docker_host.clone();

    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(["compose", "-f", file_path.to_str().unwrap(), "-p", &project_name, "up", "-d"]);
    
    if !host.is_empty() {
        cmd.env("DOCKER_HOST", host);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let app_clone1 = app.clone();
    let mut stdout_reader = BufReader::new(stdout).lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            let _ = app_clone1.emit("compose-progress", line);
        }
    });

    let app_clone2 = app.clone();
    let mut stderr_reader = BufReader::new(stderr).lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            let _ = app_clone2.emit("compose-progress", line);
        }
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;

    if status.success() {
        Ok("Launched successfully".to_string())
    } else {
        Err("Failed to launch compose".to_string())
    }
}

#[tauri::command]
async fn get_system_df(state: State<'_, AppState>) -> Result<Vec<Value>, String> {
    let host = state.config.lock().unwrap().docker_host.clone();
    let mut cmd = tokio::process::Command::new("docker");
    cmd.args(["system", "df", "--format", "{{json .}}"]);
    if !host.is_empty() {
        cmd.env("DOCKER_HOST", host);
    }
    let output = cmd.output().await.map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();
    for line in stdout.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            results.push(v);
        }
    }
    Ok(results)
}

#[tauri::command]
async fn execute_prune(state: State<'_, AppState>, target: String) -> Result<String, String> {
    let host = state.config.lock().unwrap().docker_host.clone();
    let mut cmd = tokio::process::Command::new("docker");
    
    match target.as_str() {
        "all" => cmd.args(["system", "prune", "-a", "--volumes", "-f"]),
        "containers" => cmd.args(["container", "prune", "-f"]),
        "images" => cmd.args(["image", "prune", "-a", "-f"]),
        "volumes" => cmd.args(["volume", "prune", "-f"]),
        "networks" => cmd.args(["network", "prune", "-f"]),
        "build_cache" => cmd.args(["builder", "prune", "-f"]),
        _ => return Err("Invalid prune target".to_string()),
    };

    if !host.is_empty() {
        cmd.env("DOCKER_HOST", host);
    }

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_config_dir().unwrap_or_else(|_| {
                std::env::temp_dir().join("edocker")
            });
            let config_path = app_data_dir.join("config.json");
            app.manage(AppState::load(config_path));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_docker_config, update_docker_config,
            get_docker_info,
            list_containers, inspect_container, container_logs,
            start_container, stop_container, restart_container,
            pause_container, unpause_container, remove_container,
            start_terminal, write_terminal, resize_terminal,
            list_images, inspect_image, pull_image, remove_image,
            list_volumes, inspect_volume, remove_volume, prune_volumes,
            list_networks, inspect_network, remove_network, create_network,
            launch_compose,
            get_system_df, execute_prune,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
