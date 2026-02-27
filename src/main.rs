use clap::{ArgGroup, Parser};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

#[derive(Parser, Debug)]
#[command(name = "nscratch")]
#[command(group(
	ArgGroup::new("selector")
		.required(false)
		.args(&["app_id", "title"]),
))]
struct Args {
	#[arg(short = 'i', long = "app-id")]
	app_id: Option<String>,

	#[arg(short = 't', long)]
	title: Option<String>,

	#[arg(short = 's', long)]
	spawn: Option<String>,

	#[arg(short = 'a', long)]
	animations: bool,

	#[arg(short = 'm', long)]
	multi_monitor: bool,

	#[arg(long)]
	mark: bool,

	#[arg(long)]
	index: Option<usize>,

	#[arg(long)]
	remove: Option<usize>,

	#[arg(long)]
	list: bool,
}

#[derive(Default, Debug)]
struct ScratchWindow {
	id: i64,
	workspace_id: i64,
	is_focused: bool,
	is_floating: bool,
	found: bool,
}

#[derive(Default, Debug)]
struct FocusedWorkspace {
	idx: i64,
	output: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ScratchEntry {
	id: i64,
	title: String,
	app_id: String,
	workspace: i64,
}

#[derive(Default, Serialize, Deserialize)]
struct ScratchState {
	windows: Vec<ScratchEntry>,
}

fn notify(msg: &str) {
	let _ = Command::new("notify-send")
		.arg("nscratch")
		.arg(msg)
		.status();
}

fn state_path() -> PathBuf {
	let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
	path.push("nscratch");
	let _ = fs::create_dir_all(&path);
	path.push("state.json");
	path
}

fn load_state() -> ScratchState {
	fs::read(state_path())
		.ok()
		.and_then(|b| serde_json::from_slice(&b).ok())
		.unwrap_or_default()
}

fn save_state(state: &ScratchState) {
	if let Ok(data) = serde_json::to_vec_pretty(state) {
		let _ = fs::write(state_path(), data);
	}
}

fn reconcile_state(state: &mut ScratchState, windows: &[Value]) {
	state.windows.retain(|entry| {
		windows.iter().any(|w| w["id"].as_i64() == Some(entry.id))
	});
}

fn fetch_focused_window_full(windows: &[Value]) -> Option<&Value> {
	windows.iter().find(|w| w["is_focused"].as_bool() == Some(true))
}

fn niri_cmd(args: &[String]) {
	let _ = Command::new("niri")
		.arg("msg")
		.arg("action")
		.args(args)
		.status();
}

fn move_window_to_scratchpad(
	window_id: i64,
	scratch_workspace: &str,
	animations: bool,
) {
	niri_cmd(&[
		"move-window-to-workspace".into(),
		"--window-id".into(),
		window_id.to_string(),
		scratch_workspace.into(),
		"--focus=false".into(),
	]);

	if animations {
		niri_cmd(&[
			"move-window-to-tiling".into(),
			"--id".into(),
			window_id.to_string(),
		]);
	}
}

fn bring_scratchpad_window_to_focus(
	window_id: i64,
	args: &Args,
	scratch_window: &ScratchWindow,
	focused_workspace: &FocusedWorkspace,
) {
	niri_cmd(&[
		"move-window-to-workspace".into(),
		"--window-id".into(),
		window_id.to_string(),
		focused_workspace.idx.to_string(),
	]);

	if args.multi_monitor {
		niri_cmd(&[
			"move-window-to-monitor".into(),
			"--id".into(),
			window_id.to_string(),
			focused_workspace.output.clone(),
		]);
	}

	if args.animations && !scratch_window.is_floating {
		niri_cmd(&[
			"move-window-to-floating".into(),
			"--id".into(),
			window_id.to_string(),
		]);
	}

	niri_cmd(&[
		"focus-window".into(),
		"--id".into(),
		window_id.to_string(),
	]);
}

fn find_scratch_window(args: &Args, windows: &[Value]) -> ScratchWindow {
	let mut scratch = ScratchWindow::default();

	for window in windows {
		let matches = match (&args.app_id, &args.title) {
			(Some(app_id), _) => window["app_id"] == *app_id,
			(_, Some(title)) => window["title"] == *title,
			_ => false,
		};

		if matches {
			scratch.id = window["id"].as_i64().unwrap();
			scratch.workspace_id = window["workspace_id"].as_i64().unwrap();
			scratch.is_focused = window["is_focused"].as_bool().unwrap();
			scratch.is_floating = window["is_floating"].as_bool().unwrap();
			scratch.found = true;
			break;
		}
	}

	scratch
}

fn fetch_focused_workspace(focused: &mut FocusedWorkspace) -> i64 {
	let output = Command::new("niri")
		.arg("msg")
		.arg("--json")
		.arg("workspaces")
		.output()
		.expect("failed to run niri");

	let workspaces: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();

	for ws in workspaces {
		if ws["is_focused"].as_bool().unwrap() {
			focused.idx = ws["idx"].as_i64().unwrap();
			focused.output = ws["output"].as_str().unwrap().to_string();
			return ws["id"].as_i64().unwrap();
		}
	}

	unreachable!("No focused workspace found");
}

fn build_scratch_from_id(id: i64, windows: &[Value]) -> ScratchWindow {
	let mut scratch = ScratchWindow::default();

	for window in windows {
		if window["id"].as_i64() == Some(id) {
			scratch.id = id;
			scratch.workspace_id = window["workspace_id"].as_i64().unwrap();
			scratch.is_focused = window["is_focused"].as_bool().unwrap();
			scratch.is_floating = window["is_floating"].as_bool().unwrap();
			scratch.found = true;
			break;
		}
	}

	scratch
}

fn notify_workspace_list(state: &ScratchState) {
	let mut message = String::from("Scratchpads:\n\n");

	for (i, entry) in state.windows.iter().enumerate() {
		message.push_str(&format!(
			"[{}] {} | {} | from workspace {}\n",
			i + 1,
			entry.app_id,
			entry.title,
			entry.workspace-1
		));
	}

	if state.windows.is_empty() {
		message.push_str("none");
	}

	notify(&message);
}

fn remove_scratchpad(
	index: usize,
	state: &mut ScratchState,
	windows: &[Value],
	args: &Args,
) -> ExitCode {
	if index == 0 || index > state.windows.len() {
		notify("No scratchpad window at this index");
		return ExitCode::from(1);
	}

	let entry = state.windows.remove(index - 1);
	save_state(state);

	let scratch_window = build_scratch_from_id(entry.id, windows);

	if !scratch_window.found {
		return ExitCode::SUCCESS;
	}

	let mut focused_workspace = FocusedWorkspace::default();
	fetch_focused_workspace(&mut focused_workspace);

	niri_cmd(&[
		"move-window-to-workspace".into(),
		"--window-id".into(),
		entry.id.to_string(),
		focused_workspace.idx.to_string(),
	]);

	niri_cmd(&[
		"move-window-to-tiling".into(),
		"--id".into(),
		entry.id.to_string(),
	]);

	if args.multi_monitor {
		niri_cmd(&[
			"move-window-to-monitor".into(),
			"--id".into(),
			entry.id.to_string(),
			focused_workspace.output.clone(),
		]);
	}

	niri_cmd(&[
		"focus-window".into(),
		"--id".into(),
		entry.id.to_string(),
	]);

	ExitCode::SUCCESS
}

fn ns(args: Args) -> ExitCode {
	let scratch_workspace =
		env::var("NS_WORKSPACE").unwrap_or_else(|_| "scratch".to_string());

	let output = Command::new("niri")
		.arg("msg")
		.arg("--json")
		.arg("windows")
		.output()
		.expect("failed to run niri");

	let windows: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();

	let mut state = load_state();
	reconcile_state(&mut state, &windows);
	save_state(&state);

	if args.list {
		notify_workspace_list(&state);
		return ExitCode::SUCCESS;
	}

	if let Some(idx) = args.remove {
		return remove_scratchpad(idx, &mut state, &windows, &args);
	}

	if args.mark {
		let Some(win) = fetch_focused_window_full(&windows) else {
			return ExitCode::from(1);
		};

		let id = win["id"].as_i64().unwrap();

		if !state.windows.iter().any(|e| e.id == id) {
			let entry = ScratchEntry {
				id,
				title: win["title"].as_str().unwrap_or("unknown").to_string(),
				app_id: win["app_id"].as_str().unwrap_or("unknown").to_string(),
				workspace: win["workspace_id"].as_i64().unwrap_or(0),
			};

			state.windows.push(entry);
			save_state(&state);
			move_window_to_scratchpad(id, &scratch_workspace, args.animations);
		}

		return ExitCode::SUCCESS;
	}

	if let Some(index) = args.index {
		if index == 0 || index > state.windows.len() {
			notify("No scratchpad window at this index");
			return ExitCode::from(1);
		}

		let window_id = state.windows[index - 1].id;

		let scratch_window = build_scratch_from_id(window_id, &windows);

		if !scratch_window.found {
			return ExitCode::from(1);
		}

		if !scratch_window.is_focused {
			let mut focused_workspace = FocusedWorkspace::default();
			let workspace_id = fetch_focused_workspace(&mut focused_workspace);

			if scratch_window.workspace_id != workspace_id {
				bring_scratchpad_window_to_focus(
					window_id,
					&args,
					&scratch_window,
					&focused_workspace,
				);
				return ExitCode::SUCCESS;
			}
		}

		move_window_to_scratchpad(
			window_id,
			&scratch_workspace,
			args.animations,
		);

		return ExitCode::SUCCESS;
	}

	let scratch_window = find_scratch_window(&args, &windows);

	if !scratch_window.found {
		if let Some(spawn) = args.spawn {
			let mut parts = spawn.split_whitespace();
			let cmd = parts.next().unwrap();
			let rest: Vec<&str> = parts.collect();

			let mut args = vec!["spawn".into(), "--".into()];
			args.push(cmd.into());
			args.extend(rest.into_iter().map(String::from));

			niri_cmd(&args);
			return ExitCode::SUCCESS;
		} else {
			eprintln!("No scratchpad window found.");
			return ExitCode::from(1);
		}
	}

	let window_id = scratch_window.id;

	if !scratch_window.is_focused {
		let mut focused_workspace = FocusedWorkspace::default();
		let workspace_id = fetch_focused_workspace(&mut focused_workspace);

		if scratch_window.workspace_id != workspace_id {
			bring_scratchpad_window_to_focus(
				window_id,
				&args,
				&scratch_window,
				&focused_workspace,
			);
			return ExitCode::SUCCESS;
		}
	}

	move_window_to_scratchpad(
		window_id,
		&scratch_workspace,
		args.animations,
	);

	ExitCode::SUCCESS
}

fn main() -> ExitCode {
	let args = Args::parse();
	ns(args)
}
