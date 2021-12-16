// Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[derive(Debug, PartialEq)]
pub struct Speed {
    multiplier: f64,
}

impl Default for Speed {
    fn default() -> Self {
        Speed::STOP
    }
}

impl Speed {
    const fn new(multiplier: f64) -> Self {
        Self { multiplier }
    }

    pub const SPRINT: Speed = Speed::new(1.3);
    pub const WALK: Speed = Speed::new(1.0);
    pub const SNEAK: Speed = Speed::new(0.3);
    pub const STOP: Speed = Speed::new(0.);

    pub fn multiplier(&self) -> f64 {
        self.multiplier * 0.98 // TODO: differnet at 45 degree angle
    }
}
