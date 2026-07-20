use std::{rc::Rc, sync::OnceLock};

use ratatui::{
    Frame,
    crossterm::event::Event,
    layout::{Constraint, Layout, Rect},
    text::Text,
    widgets::Block,
};
use tachyonfx::fx::{self};

use crate::ui::{
    RectExt,
    action::Action,
    animations::{Animator, logo::LogoAnimations},
    component::Component,
    components::{
        help::{HelpComponent, HelpItem},
        menu::{MenuComponent, MenuItem},
    },
    config::Config,
    keymap::{GLOBAL_KEYMAP, MENU_KEYMAP},
    theme::Theme,
};

const GAP: u16 = 3;
const GAP_MEDIUM: u16 = 2;
const GAP_SMALL: u16 = 1;
const MENU_ITEMS: &[MenuItem<MenuAction>] = &[
    MenuItem {
        label: "New Random Game",
        action: MenuAction::NewRandomGame,
    },
    MenuItem {
        label: "New Solvable",
        action: MenuAction::NewSolvableGame,
    },
    MenuItem {
        label: "Quit",
        action: MenuAction::Quit,
    },
];
const HELP_ITEMS: &[HelpItem] = &[
    HelpItem::new(
        &[MENU_KEYMAP.previous.label(), MENU_KEYMAP.next.label()],
        "select",
    ),
    HelpItem::new(&[MENU_KEYMAP.confirm.label()], "activate"),
    HelpItem::new(&[GLOBAL_KEYMAP.quit.label()], "quit"),
];

fn logo() -> &'static Text<'static> {
    static LOGO: OnceLock<Text> = OnceLock::new();
    LOGO.get_or_init(|| {
        Text::from(
            r#"███████╗ ██████╗ ██╗     ██╗████████╗ █████╗ ██╗██████╗ ███████╗
██╔════╝██╔═══██╗██║     ██║╚══██╔══╝██╔══██╗██║██╔══██╗██╔════╝
███████╗██║   ██║██║     ██║   ██║   ███████║██║██████╔╝█████╗  
╚════██║██║   ██║██║     ██║   ██║   ██╔══██║██║██╔══██╗██╔══╝  
███████║╚██████╔╝███████╗██║   ██║   ██║  ██║██║██║  ██║███████╗
╚══════╝ ╚═════╝ ╚══════╝╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝╚══════╝
"#,
        )
    })
}

fn logo_small() -> &'static Text<'static> {
    static LOGO: OnceLock<Text> = OnceLock::new();
    LOGO.get_or_init(|| {
        Text::from(
            "┏━┓┏━┓╻  ╻╺┳╸┏━┓╻┏━┓┏━╸
┗━┓┃ ┃┃  ┃ ┃ ┣━┫┃┣┳┛┣╸ 
┗━┛┗━┛┗━╸╹ ╹ ╹ ╹╹╹┗╸┗━╸",
        )
    })
}

fn logo_medium() -> &'static Text<'static> {
    static LOGO: OnceLock<Text> = OnceLock::new();
    LOGO.get_or_init(|| {
        Text::from(
            r#"▄█████  ▄▄▄  ▄▄    ▄▄ ▄▄▄▄▄▄ ▄▄▄  ▄▄ ▄▄▄▄  ▄▄▄▄▄
▀▀▀▄▄▄ ██▀██ ██    ██   ██  ██▀██ ██ ██▄█▄ ██▄▄ 
█████▀ ▀███▀ ██▄▄▄ ██   ██  ██▀██ ██ ██ ██ ██▄▄▄"#,
        )
    })
}

#[derive(Clone, Copy)]
enum MenuAction {
    NewRandomGame,
    NewSolvableGame,
    Quit,
}

pub(crate) struct MainMenu {
    animator: Animator,
    theme: Rc<Theme>,
    help: HelpComponent,
    menu: MenuComponent<MenuAction>,
}

impl MainMenu {
    pub(crate) fn new(config: Rc<Config>, theme: Rc<Theme>) -> Self {
        let mut animator = Animator::new(config.disable_animations);
        animator.add_effect(LogoAnimations::random_transition());
        animator.add_effect(fx::repeating(LogoAnimations::idle(theme.shine_color)));

        let menu = MenuComponent::new(theme.menu_items.clone(), MENU_ITEMS);
        let help = HelpComponent::new(theme.menu_help.clone(), HELP_ITEMS);

        Self {
            animator,
            theme,
            menu,
            help,
        }
    }
}

impl Component<Action> for MainMenu {
    fn update(&mut self, event: Event) -> Option<Action> {
        match event {
            Event::Key(_) | Event::Mouse(_) => self.menu.update(event).map(|action| match action {
                MenuAction::NewRandomGame => Action::NewRandomGame,
                MenuAction::NewSolvableGame => Action::NewSolvableGame,
                MenuAction::Quit => Action::Quit,
            }),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Block::default().style(self.theme.menu), area);

        // logo
        let [main_area, help_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let (logo, gap) = get_logo(area.width);
        let logo_height = logo.height() as u16;
        let menu_height = MENU_ITEMS.len() as u16;

        let cluster_height = logo_height + gap + menu_height;
        let cluster_area = main_area.centered_vertically(Constraint::Length(cluster_height));

        let [logo_area, _, menu_area] = Layout::vertical([
            Constraint::Min(logo_height),
            Constraint::Length(gap),
            Constraint::Fill(1),
        ])
        .areas(cluster_area);

        frame.render_widget(logo.style(self.theme.logo).centered(), logo_area);

        self.animator.process_effects(frame.buffer_mut(), logo_area);

        // menu options
        self.menu.render(frame, menu_area);

        // help
        let help_height = self.help.height(help_area.width);
        let help_area = help_area.resize_height_from_top(help_height);
        self.help.render(frame, help_area);
    }
}

fn get_logo(width: u16) -> (Text<'static>, u16) {
    if width >= logo().width() as u16 {
        (logo().clone(), GAP)
    } else if width >= logo_medium().width() as u16 {
        (logo_medium().clone(), GAP_MEDIUM)
    } else {
        (logo_small().clone(), GAP_SMALL)
    }
}
