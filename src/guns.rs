use crate::gfx;
use core::iter::Chain;
use core::slice;

pub const REVOLVER_OFFSET: u8 = 7;
pub const SCORPIO_OFFSET: u8 = 8;

pub const MAX_GUARANTEED_REACH: u8 = gfx::min(REVOLVER_OFFSET, SCORPIO_OFFSET);

pub enum Gun<'a> {
    Revolver(&'a mut Revolver),
    Scorpio(&'a mut Scorpio),
}

impl<'a> Gun<'a> {
    pub fn shoot(&mut self) -> Option<(bool, u8)> {
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
            chambers: [
                Chamber::Empty,
                Chamber::Loaded,
                Chamber::Loaded,
                Chamber::Loaded,
                Chamber::Loaded,
                Chamber::Loaded,
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

    pub fn shoot(&mut self) -> (bool, u8) {
        self.drum_clockwise();
        match self.chambers().next() {
            Some(Chamber::Empty) => (),
            Some(Chamber::Loaded) => {
                self.set_chamber(Chamber::Shot);
                return (true, REVOLVER_OFFSET);
            }
            Some(Chamber::Shot) => (),
            None => (),
        }
        (false, REVOLVER_OFFSET)
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
    pub const FIRE_RATE: u8 = 3;

    pub fn new() -> Self {
        Self { rounds: 20 }
    }

    pub fn shoot(&mut self) -> Option<(bool, u8)> {
        if self.rounds > 0 {
            self.rounds -= 1;
            Some((true, SCORPIO_OFFSET))
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
