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
use rand::Rng;
use crate::{*};

pub struct Grid {
    pub x: usize,
    pub y: usize,
    pub bins: Vec<Bin>,
    pub mapping: Vec<usize>,
    pub edges: Vec<Edge>,
} 

pub struct Bin {
    pub x: f32,
    pub y: f32,
    pub grid_x: usize,
    pub grid_y: usize,
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
    pub fn gi(&self, x: usize, y: usize) -> usize {
        (y * self.x) + x
    }
    pub fn new(x: usize, y: usize)-> Grid {
        let count = (x * y) as usize;
        let mut g = Grid {
            x,
            y,
            bins: Vec::with_capacity(count),
            mapping: vec!(0 as usize; count),
            edges: Vec::new(),
        };

        for y in 0..y {
            for x in 0..x {
                let idx = g.gi(x, y);
                g.bins.push(Bin { x: x as f32, y: y as f32, grid_x: x, grid_y: y, index: idx });
                g.mapping[idx] = idx;
            }
        }

        g
    }
    pub fn add_edge(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, weight: f32) {
        let idx1 = self.gi(x1, y1);
        let idx2 = self.gi(x2, y2);
        if (idx1 < idx2) {
            self.edges.push(Edge { a: idx1, b: idx2, weight});
        } else {
            self.edges.push(Edge { a: idx2, b: idx1, weight});
        }

    }
    pub fn grid_cost(&self) -> f32 {
        let mut total = 0.0;
        for e in &self.edges {
            // Edge goes from cluster a to cluster b -- so get the coords for each, multiply
            // by the weight of the edge
            let x1 = self.bins[self.mapping[e.a]].x;
            let y1 = self.bins[self.mapping[e.a]].y;
            let x2 = self.bins[self.mapping[e.b]].x;
            let y2 = self.bins[self.mapping[e.b]].y;
            let c = ((x1 - x2).abs() + (y1 - y2).abs()) * e.weight;
            println!("Edge from cell index {}:b{} to {}:b{} == {}", e.a, self.mapping[e.a], e.b, self.mapping[e.b], c);
            total = total + c;
        }

        total
    }

    pub fn print(&self) {
        println!("{} x {} grid", self.x, self.y);
        let mut inverse_map = vec!(0 as usize; self.bins.len());
        for i in 0..self.mapping.len() {
            inverse_map[i] = self.mapping[i];
        }
        for y in 0..self.y {
            for x in 0..self.x {
                let bin = self.gi(x, y);
                print!("{:3}[{:3}] ", self.mapping[bin], bin);
            }
            println!("");
        }
        println!("Total cost: {}", self.grid_cost());
    }
    fn pull_random(&self, list: &mut Vec<usize>) -> usize {
        let mut rng = rand::thread_rng();

        let mut idx: usize = rng.gen();
        idx = idx % list.len();
        let n = list[idx];
        list[idx] = *list.last().unwrap();
        list.pop();
        
        n
    }
    pub fn scramble(&mut self, llx: usize, lly: usize, urx: usize, ury: usize) {
        let mut list = Vec::new();

        for x in llx..urx {
            for y in lly..ury {
                let idx = self.gi(x, y);
                list.push(idx);
            }
        }
        for x in llx..urx {
            for y in lly..ury {
                let idx = self.gi(x, y);
                let v = self.pull_random(&mut list);
                self.mapping[idx] = v;
            }
        }        
    }

    pub fn opt_h(&mut self, llx: usize, lly: usize, urx: usize, ury: usize) {
        
        println!("Optimize {} strips of {} tall", urx - llx, ury - lly);
        // HCWT Root Context
        let c = Context {
            base_cost: 0.0,
            mapping: self.mapping.clone(),
        };
        // HCWT ROOT NODES
        let root = Node {
            cost: 0.0,
            selection: 0,
        };
        let mut roots = Vec::new();
        roots.push(root);
        // HCWT ROOT DECISION  (Placeholder)
        let root_dec = Decision {
            cells: Vec::new(),
            mappings: Vec::new(),
        };
        let mut decs = Vec::new();
        decs.push(root_dec);
        let mut hcwt: HCwT<Context,Decision,Node> = HCwT::new(c, decs, roots);


        // Now construct the levels        
        let mut inverse_map = vec!(0 as usize; self.bins.len());
        for i in 0..self.mapping.len() {
            inverse_map[i] = self.mapping[i];
        }

        for x in llx..urx {
            let mut dx = hcwt.new_decision();
            let mut decision = Decision {
                cells: Vec::new(),
                mappings: Vec::new(),
            };
            println!("Strip at X: {}", x);
            let mut orig_map = Vec::new();
            for y in lly..ury {
                let idx = self.gi(x, y);
                decision.cells.push(self.mapping[idx]);
                orig_map.push(idx);
                println!("  Original mapping cell {} to bin {}", self.mapping[idx], idx);
            }
            let permlist = perms(orig_map.len() as u32, 100);
            for p in &permlist {
                println!("New mapping");
                let mut new_map = Vec::new();
                for i in &p.items {
                    new_map.push(orig_map[*i]);
                    println!("  Bin {}", orig_map[*i]);
                }
                decision.mappings.push(new_map);
            }
            // Add options for each permutation            
            hcwt.add_option(&mut dx, decision);
            hcwt.add_decision(dx);
        }

       

        hcwt.solve(&generate, &filter);

        println!("Solved ----");
        let last = &hcwt.levels.last().unwrap().nodes;
        let level = hcwt.levels.len() - 1;
        for i in 0..last.len() {
            let node = &last[i];
            println!("----- SOLUTION {}----", i);
            trace(&hcwt, level, i);
        }

    }


}

pub fn trace(hcwt: &HCwT<Context,Decision,Node>, level: usize, index: usize) {

    if level > 0 {
        trace(hcwt, level - 1, hcwt.levels[level].nodes[index].parent);
    }
    let node = &hcwt.levels[level].nodes[index].detail;
    if level >= 0 {
        println!("{} << Selection cost {}, parent {}", node.selection, node.cost, hcwt.levels[level].nodes[index].parent);
        println!("  Level {}", level);
        if level < hcwt.levels.len() {
            let d = &hcwt.levels[level].decisions[0];
            for i in 0..d.cells.len() {
                println!("   Cell {} maps to bin {}", d.cells[i], d.mappings[node.selection][i]);
                // println!("   Cell {} maps to bin ..", d.cells[i]);
            }
    }
    }
}


fn generate(hcwt: &HCwT<Context,Decision,Node>, level: usize, decision_index: usize, parent_node_index: usize, nodes: &mut Vec<SelectionNode<Node>>) {
    println!("Generate for level {}", level);
    let detail = &hcwt.levels[level].decisions[decision_index];
    let parent_node = &hcwt.levels[level - 1].nodes[parent_node_index];
    for mi in 0..detail.mappings.len() {
        for i in 0..detail.cells.len() {
            println!("Generate map cell {} to bin {}", detail.cells[i], detail.mappings[mi][i]);
        }
        let mut new_node = parent_node.detail;
        new_node.selection = mi;
        nodes.push(SelectionNode{parent: parent_node_index, detail: new_node});
    }
}


fn filter(hcwt: &HCwT<Context,Decision,Node>, level: usize, nodes: &mut Vec<SelectionNode<Node>>) -> Vec<SelectionNode<Node>>
{
    let mut result = nodes.clone();

    result
}

fn display(hcwt: &HCwT<Context,Decision,Node>, dec: Decision, context: &mut Context) {

}

struct Context {
    base_cost: f32,
    mapping: Vec<usize>,
}
struct Decision {
    cells: Vec<usize>,
    mappings: Vec<Vec<usize>>,
}

#[derive(Copy, Clone)]
struct Node {
    cost: f32,
    selection: usize,
}

pub fn grid_test() {
    let mut bb = pstools_r::bbox::BBox::new();
    bb.addpoint(3.0, 4.0);
    let xdim = 7;
    let ydim = 4;

    let mut g = Grid::new(xdim, ydim);
    for x in 0..xdim {
        for y in 0..(ydim - 1) {
            g.add_edge(x, y, x, y + 1, 1.0);
        }
    }
    for y in 0..ydim {
        for x in 0..(xdim - 1) {
            g.add_edge(x, y, x + 1, y, 1.0);
        }
    }
    
    println!("Grid: ");
    g.print();
    g.scramble(1, 1, xdim - 1, ydim - 1);
    g.print();

    // Scramble the bin assignments

    // Generate the HCwT -- horizontal layers.
    g.opt_h(1, 1, xdim - 1, ydim - 1);

}
