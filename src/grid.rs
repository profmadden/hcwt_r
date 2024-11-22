use std::sync::Arc;

// Rearrangement of grid-mapped items
//
// We get an x by y grid, and have a bunch of items located on that
// grid pattern, plus 2X + 2Y + 4 propagated locations.
// All connections are point-to-point to grid locations ().
//
// Maybe make the actual positions (X+2) by (Y + 2)?  Then
// simple indexing, the internals start at (1,1).  Hmmm.
//
// We have permutations that pick the items in each row or
// column, as a series of decision for the HCwT engine.
// Generate permutations, where each cell gets mapped to
// a different bin, and on each level, we have a list of
// point-to-point connections to add to a running total.
//
// Cells come in with initial bin assignments (and we have
// a total cost for the entire solution).  Build the
// HCWT, run a pass, and then reassign the cells to different
// bins.
//
// A pass over the grid would have direction (X or Y),
// potentially an ordering of left-to-right or right-to-left,
// The width of each layer (might do 6 cells in a 8-wide grid),
// a step size if there's overlap, and a delta on allowed
// permutations.
//
// If we have a 2x2 grid, the actual bin is 4x4
//  p p p p
//  p A B p
//  p C D p
//  p p p p
//
// The cost/weight is between original grid locations,
// and the rearrangement is mapping {ABCD} to new
// XY locations in the grid.  Each bin will have the
// grid number, the coordinate, and the XY spot to map to.
//
// When calculating the cost of a connection, it
// will either be from a cell that gets fixed in the
// current level, to either a cell in a prior level,
// or to a cell in a propagated bin.
// Make two lists of connections that get computed at each
// level -- and then scan accordingly.
// There's the cost of the cells at this level to the
// outside world, and that gets set.  Then, consider
// each of connections to stuff from the solution tree.
//
// Probably overthinking this -- it should be plenty fast
// enough.  
use pstools_r;

pub struct Grid {
    pub x: u32,
    pub y: u32,
    pub bins: Vec<Bin>,
    pub edges: Vec<Edge>,
} 

pub struct Bin {
    pub x: f32,
    pub y: f32,
    pub grid_x: u32,
    pub grid_y: u32,
    pub index: usize,
}

pub struct Cell {
    pub tag: usize,
    pub grid_x: u32,
    pub grid_y: u32,
}

pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub weight: f32,
}

pub struct Permlist {
    pub items: Vec<usize>,
}


pub fn gen_permutation(items: &Vec<usize>, taken: &mut Vec<bool>, decided: &mut Vec<usize>, delta: u32, max_delta: u32, order: &mut Vec<Permlist>) {
    if delta > max_delta {
        return
    }
    
    if decided.len() == items.len() {
        order.push(Permlist {
            items: decided.clone(),
        });        
    } else {
        for i in 0..items.len() {
            if !taken[i] {
                let i1 = i as i32;
                let i2 = decided.len() as i32;
                let increase = (i2 - i1).abs() as u32;

                // Take item i, put it into the decided list, recurse
                decided.push(items[i]);
                taken[i] = true;
                
                gen_permutation(items, taken, decided, delta + increase, max_delta, order);
                // Remove this element
                taken[i] = false;
                decided.pop();
            }
        }
    }
}

pub fn perms(n: u32, max_delta: u32) -> Vec<Permlist> {
    let mut items = Vec::new();
    let mut taken = Vec::new();
    let mut order = Vec::new();
    let mut decided = Vec::new();
    for i in 0..n {
        items.push(i as usize);
        taken.push(false);
    }
    gen_permutation(&items, &mut taken, &mut decided, 0, max_delta, &mut order);

    order
}

pub fn print_permutations(order: &Vec<Permlist>) {
    println!("{} permutations", order.len());
    for p in order {
        let mut tot_delta = 0;
        print!("Permutation:");
        for i in 0..p.items.len() {
            print!(" {}", p.items[i]);
            let delta = ((i as i32) - (p.items[i] as i32)).abs();
            tot_delta = tot_delta + delta;
        }
        println!(" delta {}", tot_delta);
    }
}
impl Grid {
    pub fn new(x: u32, y: u32)-> Grid {
        Grid {
            x,
            y,
            bins: Vec::with_capacity(((x + 2)*(y + 2)) as usize),
            edges: Vec::with_capacity(((x + 2)*(y + 2)*(x + 2)*(y + 2)) as usize),
        }
    }
}


pub fn grid_test() {
    let mut bb = pstools_r::bbox::BBox::new();
    bb.addpoint(3.0, 4.0);
    let xdim = 6;
    let ydim = 6;

    for x in 0..xdim {
        for y in 0..ydim {
            // Create the bin, and add edges from the bin to 
            // neighbor spots

        }
    }

    // Scramble the bin assignments

    // Generate the HCwT -- horizontal layers.

}
