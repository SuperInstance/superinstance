//! Evolution - The Breeding Pipeline
//! 
//! The Evolution module handles:
//! - Tracking agent genealogy (Stud Book)
//! - Breeding new adapters (LoRA merging)
//! - Night School (scheduled improvement)
//! - Culling underperformers

mod stud_book;
mod breeding;
mod night_school;

pub use stud_book::*;
pub use breeding::*;
pub use night_school::*;
