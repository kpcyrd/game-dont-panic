use core::iter::Chain;
use core::slice;
use fugit::Duration;

pub const START_Y: u8 = 18;
const MAX_Y: u8 = 34;
const STEP_Y: u8 = 2;

// this is not 100ms, I don't know how to configure this properly
pub const TICK_INTERVAL: Duration<u64, 1, 8> = Duration::<u64, 1, 8>::millis(100);
pub const DEBOUNCE_TICKS: u8 = 1;
pub const ROUND_TICKS: u8 = 10;

pub enum Direction {
    Clockwise,
    CounterClock,
}

pub enum Gun<'a> {
    Revolver(&'a mut Revolver),
    Scorpio(&'a mut Scorpio),
}

impl<'a> Gun<'a> {
    pub fn shoot(&mut self) -> Option<bool> {
        match self {
            Gun::Revolver(gun) => Some(gun.shoot()),
            Gun::Scorpio(gun) => gun.shoot(),
        }
    }
}

pub struct Revolver {
    chambers: [Chamber; 6],
    drum_cursor: u8,
}

impl Revolver {
    pub fn new() -> Self {
        Self {
            // chambers: [Chamber::Loaded; 6],
            chambers: [
                Chamber::Loaded,
                Chamber::Loaded,
                Chamber::Empty,
                Chamber::Empty,
                Chamber::Empty,
                Chamber::Shot,
            ],
            drum_cursor: 0,
        }
    }

    /// Create an iterator that walks over all chambers, in order, starting at the cursor
    pub fn chambers(&self) -> Chain<slice::Iter<'_, Chamber>, slice::Iter<'_, Chamber>> {
        self.chambers[(self.drum_cursor as usize)..]
            .iter()
            .chain(&self.chambers[..(self.drum_cursor as usize)])
    }

    fn set_chamber(&mut self, chamber: Chamber) {
        self.chambers[self.drum_cursor as usize] = chamber;
    }

    pub fn drum_clockwise(&mut self) {
        self.drum_cursor += 1;
        self.drum_cursor %= self.chambers.len() as u8;
    }

    pub fn drum_counterclock(&mut self) {
        self.drum_cursor += (self.chambers.len() - 1) as u8;
        self.drum_cursor %= self.chambers.len() as u8;
    }

    pub fn shoot(&mut self) -> bool {
        self.drum_clockwise();
        match self.chambers().next() {
            Some(Chamber::Empty) => (),
            Some(Chamber::Loaded) => {
                self.set_chamber(Chamber::Shot);
                return true;
            }
            Some(Chamber::Shot) => (),
            None => (),
        }
        false
    }

    pub fn reload(&mut self) {
        match self.chambers().next() {
            Some(Chamber::Empty) => {
                self.set_chamber(Chamber::Loaded);
            }
            Some(Chamber::Loaded) => (),
            Some(Chamber::Shot) => {
                self.set_chamber(Chamber::Empty);
            }
            None => (),
        }
    }
}

pub struct Scorpio {
    rounds: u8,
}

impl Scorpio {
    pub fn new() -> Self {
        Self {
            // Stats taken from GTA Vice City Stories
            rounds: 50,
        }
    }

    pub fn shoot(&mut self) -> Option<bool> {
        if self.rounds > 0 {
            self.rounds -= 1;
            Some(true)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub enum Chamber {
    Empty,
    Loaded,
    Shot,
}

pub enum Button {
    ReloadToggle,
    Shoot,
}

pub enum Action {
    Rotate(Direction),
    Press(Button),
    Release(Button),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Start,
    Normal,
    Reload,
}

pub struct Game {
    screen: Screen,
    primary_gun: Option<Scorpio>,
    secondary_gun: Revolver,
    score: u32,
    y: u8,

    reload_toggle_debounce: u8,
    shoot_debounce: u8,
    round_ticks: u8,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            screen: Screen::Start,
            primary_gun: None,
            secondary_gun: Revolver::new(),
            score: 0,
            y: START_Y,

            reload_toggle_debounce: 0,
            shoot_debounce: 0,
            round_ticks: 0,
        }
    }
}

impl Game {
    pub fn new() -> Game {
        Game::default()
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn gun(&mut self) -> Gun<'_> {
        self.primary_gun
            .as_mut()
            .map(Gun::Scorpio)
            .unwrap_or(Gun::Revolver(&mut self.secondary_gun))
    }

    pub fn chambers(&self) -> Chain<slice::Iter<'_, Chamber>, slice::Iter<'_, Chamber>> {
        self.secondary_gun.chambers()
    }

    pub fn shoot(&mut self) {
        match self.gun().shoot() {
            // did fire
            Some(true) => {
                self.add_score(1);
            }
            // did not fire (but gun is not used up)
            Some(false) => (),
            // primary weapon is used up
            None => {
                self.primary_gun = None;
            }
        }
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn add_score(&mut self, points: u32) {
        self.score = self.score.saturating_add(points);
        if self.primary_gun.is_some() {
            return;
        }

        // bonus weapon drop
        if self.score % 10 == 0 {
            self.primary_gun = Some(Scorpio::new());
        }
    }

    pub fn tick(&mut self) {
        if self.reload_toggle_debounce > 0 {
            self.reload_toggle_debounce -= 1;
        }

        if self.shoot_debounce > 0 {
            self.shoot_debounce -= 1;
        }

        if self.screen != Screen::Start {
            if self.round_ticks > 0 {
                self.round_ticks -= 1;
            } else {
                self.round_ticks = ROUND_TICKS;

                // TODO: the game executes one tick
                // self.add_score(1);
            }
        }
    }

    fn debounce_for_button(&mut self, button: &Button) -> &mut u8 {
        match button {
            Button::ReloadToggle => &mut self.reload_toggle_debounce,
            Button::Shoot => &mut self.shoot_debounce,
        }
    }

    pub fn action(&mut self, action: &Action) {
        // debouncing
        match action {
            Action::Press(button) => {
                let debounce = self.debounce_for_button(button);
                if *debounce > 0 {
                    return;
                }
            }
            Action::Release(button) => {
                let debounce = self.debounce_for_button(button);
                *debounce = DEBOUNCE_TICKS;
            }
            _ => (),
        }

        // process action
        match (self.screen, action) {
            // start screen
            (Screen::Start, Action::Press(Button::Shoot)) => {
                *self = Game {
                    screen: Screen::Normal,
                    ..Default::default()
                };
            }
            (Screen::Start, _) => {}
            // default screen
            (Screen::Normal, Action::Rotate(Direction::Clockwise)) => {
                if self.y + STEP_Y <= MAX_Y {
                    self.y += STEP_Y;
                }
            }
            (Screen::Normal, Action::Rotate(Direction::CounterClock)) => {
                self.y = self.y.saturating_sub(STEP_Y);
            }
            (Screen::Normal, Action::Press(Button::ReloadToggle)) => {
                // only the revolver can be reloaded
                match self.gun() {
                    Gun::Revolver(_) => {
                        self.screen = Screen::Reload;
                    }
                    Gun::Scorpio(_) => (),
                }
            }
            (Screen::Normal, Action::Press(Button::Shoot)) => {
                self.shoot();
            }
            // reload screen
            (Screen::Reload, Action::Rotate(Direction::Clockwise)) => {
                self.secondary_gun.drum_clockwise();
            }
            (Screen::Reload, Action::Rotate(Direction::CounterClock)) => {
                self.secondary_gun.drum_counterclock();
            }
            (Screen::Reload, Action::Press(Button::Shoot)) => {
                self.secondary_gun.reload();
            }
            (Screen::Reload, Action::Press(Button::ReloadToggle)) => {
                self.screen = Screen::Normal;
            }
            // misc
            (_, Action::Release(_)) => (),
        }
    }
}