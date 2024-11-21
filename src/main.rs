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
        maxlen: 6,
    };
    let mut ferry_problem: hcwt_r::HCwT<FerryContext,FerryDecision,FerryNode> = hcwt_r::HCwT::new(fc);

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

    let mut roots = Vec::new();
    roots.push(ferry_root);
    ferry_problem.solve(roots, &generate, &filter, &display, 0 as usize);


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
    pub maxlen: u32,
}

pub fn generate(hcwt: &HCwT<FerryContext,FerryDecision,FerryNode>, context: &FerryContext, level: usize, detail: &FerryDecision, parent_index: usize, parent_node: &SelectionNode<FerryNode>,
    nodes: &mut Vec<SelectionNode<FerryNode>>)
{
    
    println!("Level {}, decision len {} left{}", level, detail.length, detail.left);
    println!(" Prior  {} {}  index {}", parent_node.detail.left_length, parent_node.detail.right_length, parent_index);
    let mut new_node = parent_node.detail;

    if detail.left {
        new_node.left_length = new_node.left_length + detail.length;
    } else {
        new_node.right_length = new_node.right_length + detail.length;
    }

    if new_node.left_length <= context.maxlen && new_node.right_length <= context.maxlen {
        nodes.push(SelectionNode{parent: parent_index, detail: new_node});
    }

}
pub fn filter(hcwt: &HCwT<FerryContext,FerryDecision,FerryNode>, nodes: &mut Vec<SelectionNode<FerryNode>>) -> Vec<SelectionNode<FerryNode>>
{
    let mut result = Vec::new();
    nodes.sort_by_key(|item|(item.detail.left_length));
    let mut prior = 99999;
    println!("Filtering.");
    for n in nodes {
        if n.detail.left_length != prior {
            if hcwt.debug != 0 {
                println!("Save element {} {}", n.detail.left_length, n.detail.right_length);
            }
            prior = n.detail.left_length;
            result.push(*n);
        } else {
            if hcwt.debug != 0 {
                println!("     Skip element {} {}", n.detail.left_length, n.detail.right_length);
            }
        }
    }
    result
}
pub fn display(hcwt: &HCwT<FerryContext,FerryDecision,FerryNode>, dec: FerryDecision, context: &mut FerryContext) {
    println!("Selection....");
}