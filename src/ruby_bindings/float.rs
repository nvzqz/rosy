use super::prelude::*;

extern "C" {
    // VALUE rb_float_new(double d)
    pub fn rb_float_new(d: f64) -> VALUE;
}
