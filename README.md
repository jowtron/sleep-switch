<p align="center"><img src="assets/icon.png" width="128" alt="Sleep Switch icon"></p>

<h1 align="center">Sleep Switch</h1>

<p align="center">A tiny macOS menu bar switch that stops your Mac sleeping, even with the lid closed.</p>

---

Sleep Switch puts a moon or a sun in your menu bar:

- **Moon**: the Mac sleeps normally.
- **Sun**: sleep is disabled. The Mac stays awake when you close the lid, when you choose Sleep from the Apple menu, and however long it sits idle.

It is a front end for `sudo pmset -a disablesleep 1` / `0`, the one setting that keeps a laptop running with the lid shut and no external display. Apps like Caffeinate and Amphetamine's default mode prevent *idle* sleep only; closing the lid still sleeps the Mac.

Built in Rust. About 14 MB of memory, no CPU to speak of, a 1 MB binary.

## Menu

| Item | What it does |
|---|---|
| **Keep Mac Awake (even with lid closed)** | Turns sleep off (sun) or back on (moon). |
| **Switch Without Password…** | Optional. Installs a rule so switching never asks for your password. See below. Once installed, the item reads **Require Password to Switch…** and removes it again. |
| **Open at Login** | Starts Sleep Switch when you log in. |
| **Quit Sleep Switch** | Quits. The sleep setting stays as it was. |

The app reads the real setting every 3 seconds, so a change made in Terminal shows up in the menu bar too.

## Switch Without Password

Changing `disablesleep` needs administrator rights, so by default each switch shows the standard macOS password dialog.

**Switch Without Password…** asks for your password once and installs `/etc/sudoers.d/sleep-switch`:

```
you ALL=(root) NOPASSWD: /usr/bin/pmset -a disablesleep 0, /usr/bin/pmset -a disablesleep 1
```

That lets your account run exactly those two commands without a password, and nothing else: not other `pmset` settings, not `sudo` in general. The file is checked with `visudo -c` before it is installed, so a bad rule can never break `sudo`. To remove it, use **Require Password to Switch…**, or `sudo rm /etc/sudoers.d/sleep-switch`.

## Install

### Download

Grab `Sleep-Switch-<version>.zip` from [Releases](../../releases), unzip it, and move **Sleep Switch.app** to Applications.

The app is not notarised by Apple, so the first launch is blocked with "Apple could not verify…". Either open **System Settings › Privacy & Security** and click **Open Anyway**, or clear the quarantine flag:

```
xattr -dr com.apple.quarantine "/Applications/Sleep Switch.app"
```

Runs on macOS 13 or later, Apple silicon and Intel.

### Build from source

Needs [Rust](https://rustup.rs).

```
git clone https://github.com/jowtron/sleep-switch
cd sleep-switch
./build.sh --open
```

`build.sh` builds a universal app (add the Intel target with `rustup target add x86_64-apple-darwin`), signs it with the first Apple Development or Developer ID identity in your keychain (ad hoc if there is none; override with `SIGNING_ID=...`), installs it to `~/Applications`, and with `--open` launches it. `--zip` also writes a release zip to `target/`.

The app icon comes from `assets/icon.svg`; `assets/make-icon.sh` regenerates `AppIcon.icns` (needs `rsvg-convert`). The menu bar icons are drawn in code in `src/icons.rs`.

## A word of caution

A Mac with sleep disabled and its lid shut keeps running at full power. In a bag, it gets hot and drains the battery. Switch back to the moon before you pack it away.

## How it works

- `src/power.rs` reads `pmset -g`, runs `sudo -n pmset -a disablesleep N` (which succeeds silently when the rule is installed), and falls back to `osascript … with administrator privileges` for the password dialog.
- `src/main.rs` is a [tao](https://github.com/tauri-apps/tao) event loop with a [tray-icon](https://github.com/tauri-apps/tray-icon) menu, set to the Accessory activation policy so it never appears in the Dock. Open at Login uses `SMAppService`.

## Licence

[MIT](LICENSE)
