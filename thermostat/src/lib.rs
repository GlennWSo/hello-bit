#![no_std]
pub mod pid;

use defmt::{dbg, info};
use nalgebra as na;
pub type M3 = na::Matrix3<f32>;
pub type V3 = na::Vector3<f32>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ThermoPart {
    temp: f32,
    capacity: Option<f32>,
}
pub type Part3 = na::Vector3<ThermoPart>;

impl ThermoPart {
    pub fn new(temp: f32, capacity: Option<f32>) -> Self {
        Self { temp, capacity }
    }
}

#[derive(Debug)]
pub struct TriThermo {
    parts: na::Vector3<ThermoPart>,
    conn: M3,
}

impl TriThermo {
    pub fn new(parts: Part3, conn: M3) -> Self {
        Self { parts, conn }
    }
    pub fn temp(&self) -> V3 {
        self.parts.map(|part| part.temp)
    }

    fn flux(&self) -> V3 {
        let temps: na::Matrix3x1<f32> = self.temp().into();
        (self.conn * temps).into()
    }

    pub fn diffuse(&mut self, dt: f32) {
        let flux = self.flux();
        for (part, heat) in self.parts.iter_mut().zip(flux.into_iter()) {
            match part.capacity {
                Some(cap) => part.temp += heat * dt / cap,
                None => continue,
            }
        }
    }
    pub fn add_heat(&mut self, dt: f32, heat: [f32; 3]) {
        for (part, heat) in self.parts.iter_mut().zip(heat.into_iter()) {
            match part.capacity {
                Some(cap) => part.temp += (heat * dt / cap),
                None => (),
            }
        }
    }

    pub fn update(&mut self, dt: f32, heat: [f32; 3]) {
        self.diffuse(dt);
        self.add_heat(dt, heat)
    }
}
