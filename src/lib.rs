// Hill Climbing with Trees engine in Rust
//
// Input is a list of decisions, where each decision has a set of
// objects that it operates on, and a set of locations that are specific
// to a particular permutation.
// 
// For example, suppose we're doing standard cell detail placement,
// and we have cell A with width 3, B with width 2, C with width 3.
// Original order would have locations 0 3 5 
// Switch A and B gives locations 2 0 5
//
// Evaluator traces up the decision list to find where A is, and then
// gets the location off of the choice for that decision.  A is still
// in position 0, B in position 1, C in position 2 -- we look in
// the location array to figure that out.
//
// Use the locations to compute costs...

// use bookshelf_r::point::Point as Point;

// The detail for a DecisionOption is a struct
// that provides the info that the cost function
// will need.  For standard cell detail placement,
// this would be the physical locations for each cell,
// under any the provided permutation.

use pstools_r;

pub struct DecisionOption<T> {
    pub detail: T,
//     pub locations: Vec<Point>,
    pub index: usize, // For telling the caller about the option
}


pub struct Selected {
    pub cost: f32,   // Cost for this particular instance
    pub choice: f32, // Choice made here
    pub parent: f32, // Index of the choice in the prior decision
}

// The number of locations in the DecisionOption should
// match the number of elements being operated on for the
// decision.  We select option N (out of multiple choices),
// and then the elements are moved to the locations of the
// specific decision option
pub struct Decision<T> {
    pub elements: Vec<usize>, // Elements being operated on
    pub options: Vec<DecisionOption<T>>,
}

pub struct HCwT<T,Aux> {
    pub decisions: Vec<Decision<T>>,
    pub aux: Aux,
}

pub struct HCwTLayer {
    pub selections: Vec<Selected>,
}
pub struct HCwTSolution {
    pub layers: Vec<HCwTLayer>,
}

// Push decisions and options 
// Create the solver, and we provide a struct
// that gives additional information to the cost function.
impl <T,A> HCwT<T,A> {
    pub fn new(a: A) -> HCwT<T,A> {
        HCwT {
            decisions: Vec::new(),
            aux: a,
        }
    }
    // Pass information on the choices to the solver, and then
    // the solver figures out how much this configuration
    // costs..  Returns that.  
    pub fn solve(&self, cost: &dyn Fn(&HCwT<T,A>, A) -> f32, level: usize) -> f32 {

        3.3
    }
}

pub fn costfunc(hcwt: &HCwT<i32, i32>, v:i32) -> f32 {
    3.3
}

pub fn hcwt_test () {
    println!("HCWT test called");
    let mut x: i32;
    x = 10;
    let mut solver: HCwT<i32, i32> = HCwT::new(x);
    solver.solve(&costfunc, 0 as usize);

    println!("Calling PSTOOLS from hcwt_r: {}", pstools_r::psversion());

}
