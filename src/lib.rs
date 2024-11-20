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

pub struct DecisionOption<DecisionDetail> {
    pub detail: DecisionDetail,
//     pub locations: Vec<Point>,
    pub index: usize, // For telling the caller about the option
}

#[derive(Clone, Copy)]
pub struct SelectionNode<NodeDetail> {
    pub cost: f32,   // Cost for this particular instance
    pub choice: f32, // Choice made here
    pub parent: usize, // Index of the choice in the prior decision
    pub detail: NodeDetail,
}

// The number of locations in the DecisionOption should
// match the number of elements being operated on for the
// decision.  We select option N (out of multiple choices),
// and then the elements are moved to the locations of the
// specific decision option
pub struct Decision<DecisionDetail> {
    pub elements: Vec<usize>, // Elements being operated on
    pub options: Vec<DecisionOption<DecisionDetail>>,
}

pub struct HCwT<DecisionDetail,UserContext,NodeDetail> {
    pub decisions: Vec<Decision<DecisionDetail>>,
    pub user_context: UserContext,
    pub solution: HCwTSolution<NodeDetail>,
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
impl <DecisionDetail,UserContext,NodeDetail> HCwT<DecisionDetail,UserContext,NodeDetail> {
    pub fn new(user_context: UserContext) -> HCwT<DecisionDetail,UserContext,NodeDetail> {
        HCwT {
            decisions: Vec::new(),
            user_context,
            solution: HCwTSolution {
                layers: Vec::new(),
            }

        }
    }

    pub fn foo(&self) {
        println!("Function foo!");
    }
    // Pass information on the choices to the solver, and then
    // the solver figures out how much this configuration
    // costs..  Returns that.  
    pub fn solve(&mut self,
                root: NodeDetail,
                 generate: &dyn Fn(&HCwT<DecisionDetail,UserContext,NodeDetail>, &UserContext, usize, &mut Vec<SelectionNode<NodeDetail>>) -> f32,
                 filter: &dyn Fn(&HCwT<DecisionDetail,UserContext,NodeDetail>, &Vec<SelectionNode<NodeDetail>>) -> Vec<SelectionNode<NodeDetail>>,
                 display: &dyn Fn(&HCwT<DecisionDetail,UserContext,NodeDetail>, DecisionDetail, &mut UserContext),
                 level: usize) -> HCwTSolution<NodeDetail> {
        let mut tree: HCwTSolution<NodeDetail> = HCwTSolution {
            layers: Vec::new(),
        };

        // First layer of the solution tree is the root node
        tree.layers.push(HCwTLayer { nodes: Vec::new() });
        tree.layers[0].nodes.push(SelectionNode {
            cost: 0.0,
            choice: 0.0,
            parent: 0,
            detail: root
        });

        for level in 0..self.decisions.len() {
            println!("Level {}, {} decisions", level, self.decisions[level].options.len());
            
            let mut new_list = Vec::new();

            // Walk through the decisions at this level, combining with
            // selections from the prior level.  Note that at level zero, there's
            // no prior
            
            println!("Level {} options {}", level, self.decisions[level].options.len());
            for opt in &self.decisions[level].options {
                for node in &tree.layers[level].nodes {
                    generate(self, &self.user_context, level, &mut new_list);
                }
            }
            tree.layers.push(HCwTLayer { nodes: new_list});
            // Then, filter everything in this level
        }
        tree
    }
    pub fn new_decision(&self) -> Decision<DecisionDetail> {
        Decision {
            elements: Vec::new(),
            options: Vec::new(),
        }
    }
    pub fn add_option(&self, d: &mut Decision<DecisionDetail>, detail: DecisionDetail) {
        d.options.push(DecisionOption {
            detail: detail,
            index: 0,
        });
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

    println!("Calling PSTOOLS from hcwt_r: {}", pstools_r::psversion());

}
