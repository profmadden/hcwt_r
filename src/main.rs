use pstools_r;
// mod lib;
use hcwt_r::{*};

fn main() {
    println!("Hello, world!  Hill climbing with trees demo");
    println!("PSTools version {}", pstools_r::psversion());
    // crate::lib::hcwt_test();
    hcwt_r::hcwt_test();
    let mut x: i32;
    x = 10;
    let mut ferry_root = FerryNode {
        left_length: 0,
        right_length: 0,
    };
    let fc = FerryContext {
        x: 33,
    };
    let mut ferry_problem: hcwt_r::HCwT<FerryDecision,FerryContext,FerryNode> = hcwt_r::HCwT::new(fc);

    let mut dx = ferry_problem.new_decision();
    ferry_problem.add_option(&mut dx, FerryDecision{length: 0, left: false});
    ferry_problem.add_decision(dx);

    let mut dx = ferry_problem.new_decision();
    ferry_problem.add_option( &mut dx, FerryDecision{length: 1, left: true});
    ferry_problem.add_option( &mut dx, FerryDecision{length: 1, left: false});
    ferry_problem.add_decision(dx);    

    let mut dx = ferry_problem.new_decision();
    ferry_problem.add_option( &mut dx, FerryDecision{length: 3, left: true});
    ferry_problem.add_option( &mut dx, FerryDecision{length: 3, left: false});
    ferry_problem.add_decision(dx);    


    let mut dx = ferry_problem.new_decision();
    ferry_problem.add_option( &mut dx, FerryDecision{length: 5, left: true});
    ferry_problem.add_option( &mut dx, FerryDecision{length: 5, left: false});
    ferry_problem.add_decision(dx);    


    let mut dx = ferry_problem.new_decision();
    ferry_problem.add_option( &mut dx, FerryDecision{length: 3, left: true});
    ferry_problem.add_option( &mut dx, FerryDecision{length: 3, left: false});
    ferry_problem.add_decision(dx);    


    ferry_problem.solve(ferry_root, &generate, &filter, &display, 0 as usize);


}

// Ferry Loading example
//
// Each decision has a car with a length
// A selection at each layer maintains the total length of each lane
// The filter function does Pareto optimization, removing duplicates

// Each level of the decision list has two choices; left or right
#[derive(Copy, Clone)]
pub struct FerryDecision {
    pub length: u32,
    pub left: bool,
}

// Each selection at any given level has the total length on
// each side -- with decision 0 giving the base case (no parent)
#[derive(Copy, Clone)]
pub struct FerryNode {
    pub left_length: u32,
    pub right_length: u32,
}

#[derive(Copy, Clone)]
pub struct FerryContext {
    pub x: i32,
}

pub fn generate(hcwt: &HCwT<FerryDecision,FerryContext,FerryNode>, context: &FerryContext, level: usize, nodes: &mut Vec<SelectionNode<FerryNode>>) -> f32
{
    
    println!("Level {}, combine with priors", level);
    nodes.push(SelectionNode{cost: 0.0, choice: 0.0, parent: 0, detail: FerryNode{left_length: 0, right_length: 0}});
    
    3.3
}
pub fn filter(hcwt: &HCwT<FerryDecision,FerryContext,FerryNode>, nodes: &Vec<SelectionNode<FerryNode>>) -> Vec<SelectionNode<FerryNode>>
{
    nodes.clone()
}
pub fn display(hcwt: &HCwT<FerryDecision,FerryContext,FerryNode>, dec: FerryDecision, context: &mut FerryContext) {
    println!("Selection....");
}