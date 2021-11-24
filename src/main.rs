#[macro_use]
extern crate penrose;

use penrose::{
    contrib::{extensions::Scratchpad, hooks::LayoutSymbolAsRootName},
    core::{
        config::Config,
        helpers::index_selectors,
        hooks::{Hook, Hooks},
        layout::{side_stack, Layout, LayoutConf},
        manager::WindowManager,
        xconnection::XConn,
    },
    logging_error_handler,
    xcb::new_xcb_backed_window_manager,
    Backward, Forward, Less, More, Result, Selector,
};
use std::process::Command;

use simplelog::{LevelFilter, SimpleLogger};

// Replace these with your preferred terminal and program launcher
const TERMINAL: &str = "alacritty";
const LAUNCHER: &str = "dmenu_run";
const PASSWD: &str = "passmenu";

pub struct StartupScript {
    path: String,
}
impl StartupScript {
    pub fn new(s: impl Into<String>) -> Self {
        Self { path: s.into() }
    }
}
impl<X: XConn> Hook<X> for StartupScript {
    fn startup(&mut self, _: &mut WindowManager<X>) -> Result<()> {
        Ok(Command::new("sh")
            .arg("-c")
            .arg(&self.path)
            .spawn()
            .map(|_| ())
            .unwrap())
    }
}

fn main() -> penrose::Result<()> {
    // Initialise the logger (use LevelFilter::Debug to enable debug logging)
    if let Err(e) = SimpleLogger::init(LevelFilter::Info, simplelog::Config::default()) {
        panic!("unable to set log level: {}", e);
    };

    let side_layout = LayoutConf {
        follow_focus: true,
        gapless: true,
        ..Default::default()
    };
    let n_main = 1;
    let ratio = 0.55;

    let layouts = vec![Layout::new(
        "[side]",
        side_layout,
        side_stack,
        n_main,
        ratio,
    )];

    let sp = Scratchpad::new(TERMINAL, 0.8, 0.8);

    let mut config_b = Config::default().builder();
    let config = config_b
        .floating_classes(vec!["pinentry-gnome3", "dunst"])
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .layouts(layouts)
        .border_px(2)
        .focused_border(0xfdfd96)
        .gap_px(0)
        .build()
        .unwrap();

    let hooks: Hooks<_> = vec![
        sp.get_hook(),
        LayoutSymbolAsRootName::new(),
        Box::new(StartupScript::new("~/scripts/launch_bar.sh")),
    ];

    let key_bindings = gen_keybindings! {
        // Program launchers
        "M-p" => run_external!(LAUNCHER);
        "M-S-Return" => run_external!(TERMINAL);
        "M-S-p" => run_external!(PASSWD);

        // Exit Penrose (important to remember this one!)
        "M-S-q" => run_internal!(exit);

        // client management
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-m" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-S-c" => run_internal!(kill_client);

        // workspace management
        "M-Tab" => run_internal!(toggle_workspace);
        "M-A-period" => run_internal!(cycle_workspace, Forward);
        "M-A-comma" => run_internal!(cycle_workspace, Backward);

        // Layout management
        "M-grave" => run_internal!(cycle_layout, Forward);
        "M-S-grave" => run_internal!(cycle_layout, Backward);
        "M-i" => run_internal!(update_max_main, More);
        "M-d" => run_internal!(update_max_main, Less);
        "M-l" => run_internal!(update_main_ratio, More);
        "M-h" => run_internal!(update_main_ratio, Less);

        "M-slash" => sp.toggle();

        refmap [ config.ws_range() ] in {
            "M-{}" => focus_workspace [ index_selectors(config.workspaces().len()) ];
            "M-S-{}" => client_to_workspace [ index_selectors(config.workspaces().len()) ];
        };

        // screen management
        "M-period" => run_internal!(cycle_screen, Forward);
        "M-comma" => run_internal!(cycle_screen, Backward);
        "M-S-period" => run_internal!(drag_workspace, Forward);
        "M-S-comma" => run_internal!(drag_workspace, Backward);
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;
    wm.grab_keys_and_run(key_bindings, map! {})
}
