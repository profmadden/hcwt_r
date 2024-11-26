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

#[derive(Clone)]
pub struct Grid {
    pub x: usize,
    pub y: usize,
    pub bins: Vec<Bin>,
    pub map_c2b: Vec<usize>,
    pub edges: Vec<Edge>,
    pub rand_value: u32,
} 

#[derive(Copy,Clone)]
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

#[derive(Copy,Clone)]
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
    pub fn nextrand(&mut self) -> u32 {
        self.rand_value = (self.rand_value % 77) * (self.rand_value % 33) | (self.rand_value << 8);

        self.rand_value
    }
    pub fn new(x: usize, y: usize)-> Grid {
        let count = (x * y) as usize;
        let mut g = Grid {
            x,
            y,
            bins: Vec::with_capacity(count),
            map_c2b: vec!(0 as usize; count),
            edges: Vec::new(),
            rand_value: 8675309,
        };

        for y in 0..y {
            for x in 0..x {
                let idx = g.gi(x, y);
                g.bins.push(Bin { x: x as f32, y: y as f32, grid_x: x, grid_y: y, index: idx });
                g.map_c2b[idx] = idx;
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
    pub fn grid_cost(&self, map:&Vec<usize>) -> f32 {
        let mut total = 0.0;

        for e in &self.edges {
            // Edge goes from cluster a to cluster b -- so get the coords for each, multiply
            // by the weight of the edge
            let ba = map[e.a];
            let bb = map[e.b];
            let x1 = self.bins[ba].x;
            let y1 = self.bins[ba].y;
            let x2 = self.bins[bb].x;
            let y2 = self.bins[bb].y;
            let c = ((x1 - x2).abs() + (y1 - y2).abs()) * e.weight;
            // println!("Edge from cell index {}:b{} to {}:b{} == {}", e.a, map[e.a], e.b, map[e.b], c);
            total = total + c;
        }

        total
    }

    

    pub fn print(&self, map: &Vec<usize>) {
        println!("{} x {} grid", self.x, self.y);
        let mut inverse_map = vec!(0 as usize; self.bins.len());
        for i in 0..map.len() {
            inverse_map[map[i]] = i;
        }
        for y in 0..self.y {
            for x in 0..self.x {
                let bin = self.gi(x, y);
                print!("{:3}[{:3}] ", inverse_map[bin], bin);
                //print!("{:3} ", inverse_map[bin]);
            }
            println!("");
        }
        println!("Total cost: {}", self.grid_cost(map));
    }
    fn pull_random(&mut self, list: &mut Vec<usize>) -> usize {
        let mut rng = rand::thread_rng();

        //let mut idx: usize =  rng.gen();
        let mut idx: usize = self.nextrand() as usize;
        idx = idx % list.len();
        let n = list[idx];
        list[idx] = *list.last().unwrap();
        list.pop();
        
        n
    }
    pub fn scramble(&mut self, llx: usize, lly: usize, urx: usize, ury: usize) {
        
        let mut inverse_map = vec!(0 as usize; self.bins.len());
        for i in 0..self.map_c2b.len() {
            inverse_map[i] = self.map_c2b[i];
        }

        let mut list_b2c = Vec::new();
        for x in llx..urx {
            for y in lly..ury {
                let idx = self.gi(x, y);
                list_b2c.push(inverse_map[idx]);
            }
        }
        for x in llx..urx {
            for y in lly..ury {
                let idx = self.gi(x, y);
                let c = self.pull_random(&mut list_b2c);
                self.map_c2b[c] = idx;
            }
        }        
    }

    // opt_layers takes in a series of bin IDs, arranged in layers.
    // Used with the grid code, cell maping of cells to bins, and use
    // an inverse map to determine which bin contains a cell.
    // From this, we know which cells are mapped to a permutation of
    // bins in a layer.  We can also determine the nets that are
    // in play, and the layer at which a net is closed.
    //
    // Moving code out of the original opt_h, to make it generic, and
    // able to be reused in a bunch of spots.
    pub fn opt_layers(&mut self, layers: Vec<Vec<usize>>, delta: u32) {
        println!("Optimize {} layers, delta {}", layers.len(), delta);


    }

    pub fn opt_h(&mut self, llx: usize, lly: usize, urx: usize, ury: usize) {
        
        println!("Optimize {} strips of {} tall", urx - llx, ury - lly);

        
        let mut inverse_map = vec!(0 as usize; self.bins.len());
        for i in 0..self.map_c2b.len() {
            inverse_map[self.map_c2b[i]] = i;
        }

        let mut cellnet = Vec::new();
        for i in 0..self.bins.len() {
            cellnet.push(Vec::new());
        }
        for i in 0..self.edges.len() {
            let e = &self.edges[i];
            cellnet[e.a].push(i);
            cellnet[e.b].push(i);
            // println!("Edge {}  connects {} to {}", i, e.a, e.b);
        }
        let mut highmark = vec![0; self.edges.len()];
        // Now determine the last level any paricular edge is touched at
        let mut level = 1;
        for x in llx..urx {
            for y in lly..ury {
                let idx = self.gi(x, y);
                // println!("MARK grid {} is cell {}  inverse (not {})", idx, inverse_map[idx], self.map_c2b[idx]);
                let cell_id = inverse_map[idx];
                for edge_id in &cellnet[cell_id]{
                    highmark[*edge_id] = level;
                    // println!("     edge {}  connect {} {}  at level {}", edge_id, self.edges[*edge_id].a, self.edges[*edge_id].b, level);
                }
            }
            level = level + 1;
        }        

        // Group the edges by levels -- level 0 is precomputed,
        // otherwise it gets attached to the level where the last coord is set.
        let mut netlevels: Vec<Vec<usize>> = vec![Vec::new(); level];
        for eidx in 0..self.edges.len() {
            netlevels[highmark[eidx]].push(eidx);
        }

        for i in 0..level {
            println!("Edges at level {}", i);
            for v in &netlevels[i] {
                println!("Edge {}", v);
            }
        }


        // HCWT Root Context
        let mut c = Context {
            base_cost: 0.0,
            mapping: self.map_c2b.clone(),
            edges: self.edges.clone(),
            bins: self.bins.clone(),
            grid: self.clone(),
        };
        let mut share_c = Context {
            base_cost: 0.0,
            mapping: self.map_c2b.clone(),
            edges: self.edges.clone(),
            bins: self.bins.clone(),
            grid: self.clone(),
        };
        // HCWT ROOT NODES
        let root = Node {
            cost: net_cost(&self.map_c2b, &netlevels[0], &self.edges, &self.bins),
            selection: 0,
        };
        let mut roots = Vec::new();
        roots.push(root);
        // HCWT ROOT DECISION  (Placeholder)
        let root_dec = Decision {
            cells: Vec::new(),
            mappings: Vec::new(),
            closenets: Vec::new(),
        };
        let mut decs = Vec::new();
        decs.push(root_dec);
        let mut hcwt: HCwT<Context,Decision,Node> = HCwT::new(c, decs, roots);

        // Now construct the levels        
        level = 1;
        for x in llx..urx {
            let mut dx = hcwt.new_decision();
            let mut decision = Decision {
                cells: Vec::new(),
                mappings: Vec::new(),
                closenets: netlevels[level].clone(),
            };
            level = level + 1;
            println!("Strip at X: {}", x);
            let mut orig_map = Vec::new();
            for y in lly..ury {
                let idx = self.gi(x, y);
                decision.cells.push(inverse_map[idx]);
                orig_map.push(idx);
                // println!("  Original mapping cell {} to bin {}", inverse_map[idx], idx);
            }
            let permlist = perms(orig_map.len() as u32, 100);
            for p in &permlist {
               //let p = &permlist.last().unwrap();
                // println!("New mapping");
                let mut new_map = Vec::new();
                for i in &p.items {
                    new_map.push(orig_map[*i]);
                    // println!("  Bin {}", orig_map[*i]);
                }
                decision.mappings.push(new_map);
            }


            // Add options for each permutation            
            hcwt.add_option(&mut dx, decision);
            hcwt.add_decision(dx);


        }

       
        // let mut tmpgrid = c;
        hcwt.solve(&mut share_c, &generate, &filter);


        println!("Solved ----");
        let last = &hcwt.levels.last().unwrap().nodes;
        let level = hcwt.levels.len() - 1;
        let mut best = 999999.0;
        let mut best_node = &last[0];

        let mut allnets = Vec::new();
        for i in 0..self.edges.len() {
            allnets.push(i);
        }
        for i in 0..last.len() {
            let node = &last[i];
            println!("----- SOLUTION {}----", i);
            let mut mapping = self.map_c2b.clone();
            trace(&hcwt, level, i, &mut mapping);
            println!("---traced mapping----");
            self.print(&mapping);

            let nn_cost = net_cost(&mapping, &allnets, &self.edges, &hcwt.user_context.bins);

            if nn_cost < best {
                best = nn_cost;
            }
            
            
            let temp = self.map_c2b.clone();
            self.map_c2b = mapping;
            self.print(&self.map_c2b);
            println!("Allnet cost: {}", nn_cost);
            println!("NODE incremental {}", node.detail.cost);


            self.map_c2b = temp;
        }

        println!("Best cost: {}", best);

    }

}

pub fn net_cost(mapping:&Vec<usize>, nets:&Vec<usize>, edges: &Vec<Edge>, bins:&Vec<Bin>) -> f32   {
    let mut total = 0.0;

    // println!("---- Net cost ----");
    for n in nets {
        // Edge goes from cluster a to cluster b -- so get the coords for each, multiply
        // by the weight of the edge
        let e = &edges[*n];
        let ba = mapping[e.a];
        let bb = mapping[e.b];
        let x1 = bins[ba].x;
        let y1 = bins[ba].y;
        let x2 = bins[bb].x;
        let y2 = bins[bb].y;
        let c = ((x1 - x2).abs() + (y1 - y2).abs()) * e.weight;
        // println!("NCEdge {:3} from cell index {}:b{} to {}:b{} == {}", *n, e.a, mapping[e.a], e.b, mapping[e.b], c);
        total = total + c;
    }
    // println!("Total for the group: {}", total);
    total 
}

fn trace(hcwt: &HCwT<Context,Decision,Node>, level: usize, index: usize, mapping: &mut Vec<usize>) {

    if level > 0 {
        trace(hcwt, level - 1, hcwt.levels[level].nodes[index].parent, mapping);
    }
    let node = &hcwt.levels[level].nodes[index].detail;
    if level > 0 {
        // println!("{} << Selection cost {}, parent {}", node.selection, node.cost, hcwt.levels[level].nodes[index].parent);
        // println!("  Level {}", level);
        // if level < hcwt.levels.len() {
            let d = &hcwt.levels[level].decisions[0];
            for i in 0..d.cells.len() {
                // println!("   Cell {} maps to bin {}", d.cells[i], d.mappings[node.selection][i]);
                // println!("   Cell {} maps to bin ..", d.cells[i]);
                //mapping[d.mappings[node.selection][i]] = d.cells[i];
                mapping[d.cells[i]] = d.mappings[node.selection][i];
            }
        //}
    }
}


fn generate(hcwt: &HCwT<Context,Decision,Node>, context: &mut Context, level: usize, decision_index: usize, parent_node_index: usize, nodes: &mut Vec<SelectionNode<Node>>) {
    // println!("Generate for level {}", level);
    let detail = &hcwt.levels[level].decisions[decision_index];
    let parent_node = &hcwt.levels[level - 1].nodes[parent_node_index];
    
    // Set up the mapping for the prior levels
    // let mut tmpmap = hcwt.user_context.mapping.clone();
    // trace(hcwt, level - 1, parent_node_index, &mut tmpmap);
    trace(hcwt, level - 1, parent_node_index, &mut context.mapping);

    let d = &hcwt.levels[level].decisions[0];
    for mi in 0..detail.mappings.len() {
        for i in 0..detail.cells.len() {
            // println!("Generate map cell {} to bin {}", detail.cells[i], detail.mappings[mi][i]);
            // tmpmap[d.cells[i]] = d.mappings[mi][i];
            context.mapping[d.cells[i]] = d.mappings[mi][i];
        }

        let mut new_node = parent_node.detail;
        new_node.selection = mi;
        // let incremental = net_cost(&tmpmap, &d.closenets, &hcwt.user_context.edges, &hcwt.user_context.bins);
        let incremental = net_cost(&context.mapping, &d.closenets, &context.edges, &context.bins);
        // if incremental != inc2 {
        //    println!("ERROR!  Mismatch on incremental!");
        // }
        new_node.cost = parent_node.detail.cost + incremental;
        // println!("Prior cost: {} increment {} now total {}", parent_node.detail.cost, incremental, new_node.cost);
        nodes.push(SelectionNode{parent: parent_node_index, detail: new_node});
    }
}


fn filter(hcwt: &HCwT<Context,Decision,Node>, context: &mut Context, level: usize, nodes: &mut Vec<SelectionNode<Node>>) -> Vec<SelectionNode<Node>>
{
    let mut result = nodes.clone();
    result.sort_by_key({|a| a.detail.cost as i32});
    result.truncate(100);



    result
}

struct Context {
    base_cost: f32,
    mapping: Vec<usize>,
    edges: Vec<Edge>,
    bins: Vec<Bin>,
    grid: Grid,
}
struct Decision {
    cells: Vec<usize>,
    mappings: Vec<Vec<usize>>,
    closenets: Vec<usize>,
}

#[derive(Copy, Clone)]
struct Node {
    cost: f32,
    selection: usize,
}

pub fn grid_test() {
    let mut bb = pstools_r::bbox::BBox::new();
    bb.addpoint(3.0, 4.0);
    let xdim = 8;
    let ydim = 5;

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
    g.print(&g.map_c2b);
    g.scramble(1, 1, xdim - 1, ydim - 1);
    g.print(&g.map_c2b);

    // Scramble the bin assignments

    // Generate the HCwT -- horizontal layers.
    g.opt_h(1, 1, xdim - 1, ydim - 1);

}
