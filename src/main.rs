use pstools_r;
// mod lib;
use hcwt_r::{*};
mod grid;

fn main() {
    println!("Hill climbing with trees demo!  Basic Ferry Loading!");
    println!("PSTools version {}", pstools_r::psversion());
    // crate::lib::hcwt_test();
    hcwt_r::hcwt_test();

    // let g = grid::Grid::new(4, 5);

    let mut ferry_root = FerryNode {
        left_length: 0,
        right_length: 0,
    };
    let fc = FerryContext {
        maxlen: 6,
    };

    let dec = FerryDecision {
        length: 0, left: false,
    };
    let mut root_dec = Vec::new();
    root_dec.push(dec);
    let mut roots = Vec::new();
    roots.push(ferry_root);
    let mut ferry_problem: hcwt_r::HCwT<FerryContext,FerryDecision,FerryNode> = hcwt_r::HCwT::new(fc, root_dec, roots);


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


    ferry_problem.solve(&generate, &filter);

    let last = &ferry_problem.levels.last().unwrap().nodes;
    let level = &ferry_problem.levels.len() - 1;

    for i in 0..last.len() {
        let node = &last[i];
        println!("Node {} {}  (parent {})", node.detail.left_length, node.detail.right_length, node.parent);
        trace(&ferry_problem, level, i);
    }

    // let order = grid::perms(4, 6);
    // grid::print_permutations(&order);
    grid::grid_test();

}


fn trace(ferry_problem: &hcwt_r::HCwT<FerryContext,FerryDecision,FerryNode>, level: usize, index: usize)
{
    if level > 0 {
        trace(ferry_problem, level - 1, ferry_problem.levels[level].nodes[index].parent);
    }
    let nd = &ferry_problem.levels[level].nodes[index].detail;
    println!("{}  {}  (parent {})", nd.left_length, nd.right_length, ferry_problem.levels[level].nodes[index].parent);

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

pub fn generate(hcwt: &HCwT<FerryContext,FerryDecision,FerryNode>, level: usize, dec_index: usize, parent_index: usize, nodes: &mut Vec<SelectionNode<FerryNode>>)
{
    let parent_node = hcwt.levels[level - 1].nodes[parent_index];
    let detail = &hcwt.levels[level].decisions[dec_index];

    if hcwt.debug != 0 {
        println!("Level {}, decision len {} left{}", level, detail.length, detail.left);
        println!(" Prior  {} {}  index {}", parent_node.detail.left_length, parent_node.detail.right_length, parent_index);
    }
    let mut new_node = parent_node.detail;

    if detail.left {
        new_node.left_length = new_node.left_length + detail.length;
    } else {
        new_node.right_length = new_node.right_length + detail.length;
    }

    if new_node.left_length <= hcwt.user_context.maxlen && new_node.right_length <= hcwt.user_context.maxlen {
        nodes.push(SelectionNode{parent: parent_index, detail: new_node});
    }

}
pub fn filter(hcwt: &HCwT<FerryContext,FerryDecision,FerryNode>, level: usize, nodes: &mut Vec<SelectionNode<FerryNode>>) -> Vec<SelectionNode<FerryNode>>
{
    let mut result = Vec::new();
    nodes.sort_by_key(|item|(item.detail.left_length));
    let mut prior = 99999;
    if hcwt.debug != 0 {
        println!("Filtering.");
    }
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