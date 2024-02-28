mod utils;
extern crate js_sys;
extern crate fixedbitset;
use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;
use std::fmt;
extern crate web_sys;
use web_sys::console;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
//  creates output for cells in universe
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

pub struct Timer<'a>{
    name: &'a str,
}
impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}
// wasm bindgen and repru8 so js can interpret cells as strings and use cells
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
impl Cell {
    fn toggle(&mut self ) {
        *self = match *self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        };
    } 

}
// wasm needs universe
#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}
// wasm needs universe functions
#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        // let _timer = Timer::new("Univers::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);
                    
                next.set(idx, match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (true, x) if x < 2 => {
                        // log!("cell at {}, {} dies", row, col);
                        false},
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (true, 2) | (true, 3) => true,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (true, x) if x > 3 => {
                        // log!("cell at {}, {} dies", row, col);
                        false},
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (false, 3) => true,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise
                });
                // REACCESSING THIS BITARRAY SLOWS LOGGING DOWN SM
                // log!("    it becomes {:?}", next.contains(idx));
                
            }
        }

        self.cells = next;
    }
    pub fn toggle_cell( &mut self, row: u32, col : u32) {
        let idx = self.get_index(row, col);
        let mut next = self.cells.clone();
        log!("toggling cell {:?}", self.cells[idx]);
        next.set(idx, match(self.cells[idx]) {
            true => false,
            false => true,
            otherwise => otherwise
        });

        self.cells = next;
        log!("cell {:?} toggled", self.cells[idx]);
    }
    pub fn add_spaceship( &mut self, row: u32, col : u32) {
        let mut next = self.cells.clone();
        let center = self.get_index(row, col);
        let top_left = self.get_index(row - 1, col - 1);
        let right = self.get_index(row, col+1);
        let bottom = self.get_index(row + 1, col);
        let top_right = self.get_index(row + 1, col - 1);

        next.set(center, match self.cells[center] {
            true => true,
            false => true
        });

        next.set(top_left, match self.cells[top_left] {
            true => true,
            false => true
        });

        next.set(top_right, match self.cells[top_right] {
            true => true,
            false => true
        });

        next.set(right, match self.cells[right] {
            true => true,
            false => true
        });

        next.set(bottom, match self.cells[bottom] {
            true => true,
            false => true
        });

        self.cells = next;
        log!("Added spaceship");

        
    }
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
    // returns cells as pointer for ez wasm manipulation
    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
    
    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 128;
        let height = 128;
        let size = (width * height) as usize;

        let mut cells = FixedBitSet::with_capacity(size);
        for i in 0..size {
            cells.set(i, js_sys::Math::random() < 0.5 || js_sys::Math::random() > 0.5)
        }

        Universe {
            width, 
            height,
            cells,
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
    fn get_index(&self, row: u32, col: u32)-> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbor_count(&self, row:u32, col:u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned(){
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_col == 0 && delta_row == 0{
                    continue;
                }
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;

                let idx = self.get_index(neighbor_row, neighbor_col);

                count += self.cells[idx] as u8
            }
        }
        count
    }

}