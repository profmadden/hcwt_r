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
//
// Solver wil need
//   CONTEXT struct -- used in the generator if needed,
//   arbitrary info.  Is owned (to handle lifetime access).  Can
//   contain a reference, if linking to something bigger.
//
//   OPTION struct -- contained in a decision, one of the things
//   we might be able to do.  For Ferry Loading, there would be
//   two options for each decision, placing the car left or right
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

use pstools_r;


#[derive(Clone, Copy)]
pub struct SelectionNode<NodeDetail> {
    pub parent: usize, // Index of the choice in the prior decision
    pub detail: NodeDetail,
}

pub struct Decision<DecisionDetail> {
        pub details: Vec<DecisionDetail>,
}

pub struct HCwT<UserContext,DecisionDetail,NodeDetail> {
    pub user_context: UserContext,
    pub decisions: Vec<Decision<DecisionDetail>>,
    pub solution: HCwTSolution<NodeDetail>,
    pub debug: u32,
}

#[derive(Clone)]
pub struct HCwTLayer<NodeDetail> {
    pub nodes: Vec<SelectionNode<NodeDetail>>,
}

#[derive(Clone)]
pub struct HCwTSolution<NodeDetail> {
    pub layers: Vec<HCwTLayer<NodeDetail>>,
}


// Push decisions and options 
// Create the solver, and we provide a struct
// that gives additional information to the cost function.
impl <UserContext,DecisionDetail,NodeDetail> HCwT<UserContext,DecisionDetail,NodeDetail> {
    pub fn new(user_context: UserContext) -> HCwT<UserContext,DecisionDetail,NodeDetail> {
        HCwT {
            user_context,
            decisions: Vec::new(),
            solution: HCwTSolution {
                layers: Vec::new(),
            },
            debug: 0,
        }
    }

        // Pass information on the choices to the solver, and then
    // the solver figures out how much this configuration
    // costs..  Returns that.  
    pub fn solve(&mut self,
                root: Vec<NodeDetail>,
                 generate: &dyn Fn(&HCwT<UserContext,DecisionDetail,NodeDetail>, &UserContext, usize, &DecisionDetail, usize, &SelectionNode<NodeDetail>, &mut Vec<SelectionNode<NodeDetail>>),
                 filter: &dyn Fn(&HCwT<UserContext,DecisionDetail,NodeDetail>, &mut Vec<SelectionNode<NodeDetail>>) -> Vec<SelectionNode<NodeDetail>>,
                 display: &dyn Fn(&HCwT<UserContext,DecisionDetail,NodeDetail>, DecisionDetail, &mut UserContext),
                 level: usize) {
        let mut tree: HCwTSolution<NodeDetail> = HCwTSolution {
            layers: Vec::new(),
        };
        self.solution = tree;

        // First layer of the solution tree is the root node
        self.solution.layers.push(HCwTLayer { nodes: Vec::new() });
        for node in root {
            self.solution.layers[0].nodes.push(SelectionNode {
                parent: 0,
                detail: node,
            });
        }

        for level in 0..self.decisions.len() {
            println!("Level {}, {} decisions", level, self.decisions[level].details.len());
            
            let mut new_list = Vec::new();

            // Walk through the decisions at this level, combining with
            // selections from the prior level.  Note that at level zero, there's
            // no prior
            
            println!("Level {} options {}", level, self.decisions[level].details.len());
            for parent_index in 0..self.decisions[level].details.len() {
                for node_index in 0..self.solution.layers[level].nodes.len() {
                    generate(self, &self.user_context, level, &self.decisions[level].details[parent_index], node_index, &self.solution.layers[level].nodes[node_index], &mut new_list);
                }
            }
            self.solution.layers.push(HCwTLayer { nodes: filter(self, &mut new_list)});
            // Then, filter everything in this level
        }
        
    }
    pub fn new_decision(&self) -> Decision<DecisionDetail> {
        Decision {
            details: Vec::new(),
        }
    }
    pub fn add_option(&self, d: &mut Decision<DecisionDetail>, detail: DecisionDetail) {
        d.details.push(detail);
    }
    pub fn add_decision(&mut self, d: Decision<DecisionDetail>) {
        self.decisions.push(d);
    }

}

pub fn generate(hcwt: &HCwT<u32, Context,u32>, context:&Context, level: usize, nodes: &mut Vec<SelectionNode<u32>>) -> f32 {
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

    println!("Calling PSTOOLS from hcwt_r: {}", pstools_r::psversion());

}
