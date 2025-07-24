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

// The detail for a Option is a struct
// that provides the info that the cost function
// will need.  For standard cell detail placement,
// this would be the physical locations for each cell,
// under any the provided permutation.
//
// Solver wil need
//   CONTEXT struct -- this is passed to the solver, and is
//   mutable.  To make a decision, or evaluate an option, some
//   information from the general context will likely be needed.
//   For ferry loading, one might want to know the length of the ferry.
//   For circuit placement, there could be a list of connecting nets.
//
//   OPTION struct -- contained in a decision, one of the things
//   we might be able to do.  For Ferry Loading, there would be
//   two options for each decision, placing the car left or right.
//   Every decision will need at least one option.  A single option
//   can generate an arbitrary number of nodes for the search tree.
//
//   NODE struct -- details about what has been decided, total
//   running cost, etc., that's in the decision tree.  For
//   ferry loading, this would be the total length on left or right.
//
//   Callback processing routines:
//   generate() is provided the decision, the tree node parent,
//   and a list to append new tree nodes to (this is the next level).
//   The generator might append multiple nodes (or none).
//
//   filter() takes a list of nodes, and then eliminates duplicates
//   or dominated solutions.  Arbitrary code here -- whatever makes sense
//   for the problem.  In large window detail placement, we might
//   just keep the k lowest cost configurations (heuristic method).
//
//   display() should format a text string for a node.  Used for
//   debugging, or printing out a solution.
//
// The solver returns a solution tree HCwTSolution, with multiple
// tree layers, each layer having a set of nodes.  Each node has
// the index of the parent node that it's derived from.
// Ignore the parent link in any node at level 0.
//
// Any given solution can be determined by tracing back up the
// tree, using the parent index.

use pstools;
pub mod grid;
pub mod analytic;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct SelectionNode<Node> {
    pub parent: usize, // Index of the choice in the prior decision
    pub detail: Node,
}

pub struct Level<Option,Node> {
        pub decisions: Vec<Option>,
        pub nodes: Vec<SelectionNode<Node>>,
}


// https://doc.rust-lang.org/std/marker/struct.PhantomData.html
pub struct HCwT<Context,Option,Node> {
    phantom_context: PhantomData<Context>, // Swap to PhantomData<Context>
    pub levels: Vec<Level<Option,Node>>,
    // pub solution: HCwTSolution<Node>,
    pub debug: u32,
}


// Push decisions and options 
// Create the solver, and we provide a struct
// that gives additional information to the cost function.
impl <Context,Option,Node> HCwT<Context,Option,Node> {
    pub fn new(decisions: Vec<Option>, roots: Vec<Node>) -> HCwT<Context,Option,Node> {
        let mut hc = HCwT {
            // user_context: context,
            phantom_context: PhantomData,
            levels: Vec::new(),
            debug: 0,
        };
        hc.levels.push(Level {
            decisions,
            nodes: Vec::new(),
        });

        for node in roots {
            hc.levels[0].nodes.push(SelectionNode {
                parent: 0,
                detail: node,
            });
        }

        hc
    }

        // Pass information on the choices to the solver, and then
    // the solver figures out how much this configuration
    // costs..  Returns that.  
    // Generate gets the HCWT struct, the level, and the parent decision index, and the parent node index.  From that,
    // it can get all the info needed to potentially generate new nodes.  Simplifies the
    // signature.
    //
    //  generate(&hcwt, level, decision_index, parent_node_index, new_list);
    //  filter(&hcwt, level, list) -> filtered_list
    pub fn solve(&mut self,
                context: &mut Context,
                 generate: &dyn Fn(&HCwT<Context,Option,Node>, &mut Context, usize, usize, usize, &mut Vec<SelectionNode<Node>>),
                 filter: &dyn Fn(&HCwT<Context,Option,Node>, &mut Context, usize, &mut Vec<SelectionNode<Node>>) -> Vec<SelectionNode<Node>>)
        {
        // Level 0 is the initialization -- so we start processing with level 1
        for level in 1..self.levels.len() {
            if self.debug != 0 {
                println!("Level {}, {} decisions", level, self.levels[level].decisions.len());
            }
            
            let mut new_list = Vec::new();

            // Walk through the decisions at this level, combining with
            // selections from the prior level.  Note that at level zero, there's
            // no prior
            
            if self.debug != 0 {
                println!("Level {} options {}", level, self.levels[level].decisions.len());
            }
            for decision_index in 0..self.levels[level].decisions.len() {
                for node_index in 0..self.levels[level - 1].nodes.len() {
                    generate(&self, context, level, decision_index, node_index, &mut new_list);
                }
            }
            self.levels[level].nodes = filter(self, context, level, &mut new_list);

            // self.solution.layers.push(HCwTLayer { nodes: filter(self, &mut new_list)});
            // Then, filter everything in this level
        }
        
    }
    pub fn new_decision(&self) -> Level<Option,Node> {
        Level {
            decisions: Vec::new(),
            nodes: Vec::new(),
        }
    }
    pub fn add_option(&self, d: &mut Level<Option,Node>, detail: Option) {
        d.decisions.push(detail);
    }
    pub fn add_decision(&mut self, d: Level<Option,Node>) {
        self.levels.push(d);
    }

    

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

//  When doing permutations -- limit to 6 or less?
pub fn perms(n: u32, max_delta: u32) -> Vec<Permlist> {
    if n > 6 {
        println!("REQUESTED {} permutations?", n);
    }
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
pub fn generate(_hcwt: &HCwT<u32, Context,u32>, _context:&Context, _level: usize, _des: &mut Vec<SelectionNode<u32>>) -> f32 {
    // nodes.push(0);
    3.3
}

pub fn filter(hcwt: &HCwT<u32, Context,u32>, nodes: &Vec<SelectionNode<u32>>) -> Vec<SelectionNode<u32>>
{
    nodes.clone()
}

pub fn display(hcwt: &HCwT<u32, Context,u32>, v: u32, ctx: &mut Context) {
    println!("Display called");
    // ctx.z = *(ctx.z) + 1;
}

pub struct Context<'a> {
    x: i32,
    y: i32,
    z: &'a i32,
}

pub fn hcwt_test () {
    println!("HCWT test called");
    let z = 999;
    /*
    let mut ctx = Context {
        x: 33,
        y: 22,
        z: &z,
    };
    let mut x: u32;
    x = 10;
    let mut solver: HCwT<u32, Context,u32> = HCwT::new(ctx);
    solver.solve(x, &generate, &filter, &display, 0 as usize);
    solver.foo();
    */

    // println!("Calling PSTOOLS from hcwt_r: {}", pstools:

}
