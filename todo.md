### TODO LIST

Environment variable & launch option overrides per game — a text field for DXVK_HUD=1, MANGOHUD=1, PROTON_NO_ESYNC=1, etc., since different games need different Proton tweaks and you already have the launch_args infrastructure to extend for this.

MangoHud / gamescope integration — a simple toggle per game to wrap the launch command with mangohud (performance overlay) or gamescope (a compositor that fixes resolution/fullscreen issues in many Proton titles). This is a one-line prefix to your existing Command::new(...) chain, and it's one of the most commonly requested quality-of-life features in Lutris/Heroic.

Playtime tracking — you already timestamp last_played; extending that to accumulate total hours played (checking process exit time, not just launch time) turns your launcher into something closer to feature-parity with Steam's own library view.

ProtonDB link per game — a button that opens protondb.com/app/<appid> (or a manual search link since your games aren't necessarily Steam App IDs) in the system browser, so users can quickly check community compatibility reports before troubleshooting themselves.