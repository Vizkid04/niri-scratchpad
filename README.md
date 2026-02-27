# niri-scratchpad

A powerful scratchpad system for **Niri** featuring both **static** and
**dynamic indexed scratchpads**.

`niri-scratchpad` brings i3-style scratchpads and modern dynamic window
stacks together:

-   Static scratchpads (app-based)
-   Dynamic numeric scratchpads
-   Automatic index management
-   Persistent state
-   Multi-monitor aware
-   Animated floating popups
-   Zero configuration beyond normal Niri rules

------------------------------------------------------------------------

# Showcase
![Demo](demo.gif)

# ✨ Features

## Static Scratchpads

Toggle specific applications using `app-id` or window title.

Examples: - music player - file manager - terminal popup - calculator

Automatically spawns if not running.

------------------------------------------------------------------------

## Dynamic Scratchpads

Mark **any focused window** and assign it a numeric slot.

-   First marked → `[1]`
-   Next → `[2]`
-   And so on...

When a window closes or is removed:

✅ indices automatically compress\
✅ ordering stays stable\
✅ no gaps ever appear

------------------------------------------------------------------------

## Persistent Metadata

Each scratchpad stores:

-   Window title
-   Application class (`app_id`)
-   Original workspace
-   Window id

State survives compositor reloads and reboots.

------------------------------------------------------------------------

# 📦 Installation

Clone and build:

``` bash
git clone https://github.com/<your-user>/niri-scratchpad
cd niri-scratchpad
cargo build --release
```

Binary will be located at:

    target/release/nscratch

(Optional)

``` bash
sudo cp target/release/nscratch /usr/local/bin/
```

------------------------------------------------------------------------

# ⚙️ Required Niri Configuration

Create a dedicated scratch workspace:

``` ini
workspace "scratch"
```

Example window rules:

``` ini
window-rule {
    match app-id="MyMusic"
    open-on-workspace "scratch"
    open-floating true
    open-maximized true
}

window-rule {
    match app-id="Yazi"
    open-on-workspace "scratch"
    open-floating true
    default-column-width { fixed 1457; }
    default-window-height { fixed 936; }
}
```

------------------------------------------------------------------------

# ⌨️ Example Keybindings

``` ini
binds {
    Alt+Y { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --app-id 'Yazi' --spawn 'gtk-launch yazi' -a"; }
    Alt+M { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --app-id 'MyMusic' --spawn 'gtk-launch MyMusic' -a"; }

    Alt+l { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --list"; }

    Alt+0 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --mark"; }

    Alt+1 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --index 1 -a"; }
    Alt+2 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --index 2 -a"; }
    Alt+3 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --index 3 -a"; }
    Alt+4 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --index 4 -a"; }

    Alt+Shift+1 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --remove 1 -a"; }
    Alt+Shift+2 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --remove 2 -a"; }
    Alt+Shift+3 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --remove 3 -a"; }
    Alt+Shift+4 { spawn-sh "/home/vizkid/.niri_scratchpad/target/release/nscratch --remove 4 -a"; }
}
```

------------------------------------------------------------------------

# 🚀 Usage

## Static Scratchpads

Toggle or spawn an application:

``` bash
nscratch --app-id "Yazi" --spawn "gtk-launch yazi" -a
```

If the window exists → toggles visibility.\
If not → launches it.

------------------------------------------------------------------------

## Mark a Dynamic Scratchpad

Mark the currently focused window:

``` bash
nscratch --mark
```

It becomes the next index automatically.

Example:

    First mark  → [1]
    Second mark → [2]
    Third mark  → [3]

------------------------------------------------------------------------

## Toggle a Scratchpad

``` bash
nscratch --index 1
```

Press again to send it back to the scratch workspace.

------------------------------------------------------------------------

## Remove a Scratchpad

``` bash
nscratch --remove 2
```

-   Window returns to current workspace
-   Remaining indices shift automatically

Example:

    Before:
    [1] terminal
    [2] firefox
    [3] editor

    Remove 2 →

    After:
    [1] terminal
    [2] editor

------------------------------------------------------------------------

## Show Scratchpad List

``` bash
nscratch --list
```

Displays notification:

    Scratchpads:

    [1] kitty | Neovim — main.rs | from workspace 3
    [2] firefox | YouTube | from workspace 5

------------------------------------------------------------------------

# 🎛 Options

  Flag           Description
  -------------- ------------------------------------------
  `--mark`       Add focused window as dynamic scratchpad
  `--index N`    Toggle scratchpad N
  `--remove N`   Remove scratchpad N
  `--list`       Show scratchpad overview
  `--app-id`     Static scratchpad selector
  `--title`      Match window title
  `--spawn`      Launch app if missing
  `-a`           Enable animations
  `-m`           Multi-monitor support

------------------------------------------------------------------------

# 🧠 Workflow Example

Typical daily flow:

1.  Open random window
2.  `Alt+0` → mark as scratchpad
3.  Jump between workspaces
4.  `Alt+1` instantly recalls it
5.  Close window → indices auto-fix

No manual management required.

------------------------------------------------------------------------

# 📁 State Location

Scratchpad state is saved at:

    ~/.cache/nscratch/state.json

Safe to delete if reset is needed.

------------------------------------------------------------------------

# 🙏 Credits

The static scratchpad and animation concepts are heavily inspired by:

https://github.com/gvolpe/niri-scratchpad/tree/main

This project extends the idea with dynamic indexed scratchpads and persistent metadata.
