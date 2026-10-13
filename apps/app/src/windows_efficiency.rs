use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, Once};
use std::time::Duration;

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_INFORMATION, ProcessPowerThrottling, SetProcessInformation,
};

static BACKGROUNDED: AtomicBool = AtomicBool::new(false);
static APPLY_LOCK: Mutex<()> = Mutex::new(());
static MONITOR: Once = Once::new();

pub fn set_webview_backgrounded(backgrounded: bool) {
    start_monitor();
    BACKGROUNDED.store(backgrounded, Ordering::Release);
    tauri::async_runtime::spawn_blocking(apply_requested_mode);
}

fn start_monitor() {
    MONITOR.call_once(|| {
        let result = std::thread::Builder::new()
            .name("webview-efficiency-monitor".to_string())
            .spawn(|| {
                loop {
                    std::thread::sleep(Duration::from_secs(4));
                    if BACKGROUNDED.load(Ordering::Acquire) {
                        apply_requested_mode();
                    }
                }
            });

        if let Err(error) = result {
            tracing::warn!(
                "Failed to start WebView2 efficiency monitor: {error}"
            );
        }
    });
}

fn apply_requested_mode() {
    let _guard = APPLY_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let backgrounded = BACKGROUNDED.load(Ordering::Acquire);

    if let Err(error) = apply_to_webview_processes(backgrounded) {
        tracing::debug!("Failed to update WebView2 efficiency mode: {error}");
    }
}

fn apply_to_webview_processes(backgrounded: bool) -> windows::core::Result<()> {
    let processes = process_snapshot()?;
    let process_ids = descendant_process_ids(&processes, std::process::id());

    for process in processes.iter().filter(|process| {
        process_ids.contains(&process.id)
            && process.name.eq_ignore_ascii_case("msedgewebview2.exe")
    }) {
        if let Err(error) =
            set_process_efficiency_mode(process.id, backgrounded)
        {
            tracing::debug!(
                process_id = process.id,
                "Failed to update a WebView2 process efficiency mode: {error}"
            );
        }
    }

    Ok(())
}

struct ProcessInfo {
    id: u32,
    parent_id: u32,
    name: String,
}

fn process_snapshot() -> windows::core::Result<Vec<ProcessInfo>> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut processes = Vec::new();

    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
        loop {
            let name_length = entry
                .szExeFile
                .iter()
                .position(|character| *character == 0)
                .unwrap_or(entry.szExeFile.len());
            processes.push(ProcessInfo {
                id: entry.th32ProcessID,
                parent_id: entry.th32ParentProcessID,
                name: String::from_utf16_lossy(&entry.szExeFile[..name_length]),
            });

            if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
                break;
            }
        }
    }

    let _ = unsafe { CloseHandle(snapshot) };
    Ok(processes)
}

fn descendant_process_ids(
    processes: &[ProcessInfo],
    root_id: u32,
) -> HashSet<u32> {
    let mut descendants = HashSet::from([root_id]);

    loop {
        let previous_count = descendants.len();
        for process in processes {
            if descendants.contains(&process.parent_id) {
                descendants.insert(process.id);
            }
        }

        if descendants.len() == previous_count {
            return descendants;
        }
    }
}

fn set_process_efficiency_mode(
    process_id: u32,
    enabled: bool,
) -> windows::core::Result<()> {
    let process = unsafe {
        OpenProcess(
            PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            process_id,
        )
    }?;
    let control_mask = PROCESS_POWER_THROTTLING_EXECUTION_SPEED
        | PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION;
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: control_mask,
        StateMask: if enabled { control_mask } else { 0 },
    };
    let result = unsafe {
        SetProcessInformation(
            process,
            ProcessPowerThrottling,
            &state as *const PROCESS_POWER_THROTTLING_STATE as *const c_void,
            size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        )
    };

    let _ = unsafe { CloseHandle(process) };
    result
}
