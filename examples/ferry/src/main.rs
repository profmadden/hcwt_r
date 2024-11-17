// Ferry loading example for HCwT engine.  We have a series
// of cars to load onto a ferry.  We know the length of each
// car, the length of the ferry, and the ferry has two lanes.
//
// We want to pack the maximum number of cars onto the ferry,
// loading them in order, and not skipping.  This is a bin
// packing sort of problem, and a classic integer dynamic
// programming example.
//
// We keep a running total of the lengths of each lane of
// the ferry, and add a car to either the left or right
// lane (getting a new total length).  If the length
// exceeds the length of the ferry, it's an invalid solution.
//
// Dynamic programming methods track the length of one
// of the lanes -- if two or more configurations have the
// same length, we can eliminate the duplicates (they're
// redundant solutions).
//
// A simple example--we have a ferry length of 6, and
// cars that are 1, 3, 5, 3 in length.  Clearly, they
// all fit, but not every permutation is legal.

use hcwt_r;

fn main() {
    println!("Hello, world!");

    hcwt_r::hcwt_test();
}
