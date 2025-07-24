use std::{fmt, io::BufReader};
use scan_fmt::scan_fmt;

use std::fs::File;
use std::io::Write;
use std::path::Path;


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
// Generate permutations, where each module gets mapped to
// a different bin, and on each level, we have a list of
// point-to-point connections to add to a running total.
//
// modules come in with initial bin assignments (and we have
// a total cost for the entire solution).  Build the
// HCWT, run a pass, and then reassign the modules to different
// bins.
//
// A pass over the grid would have direction (X or Y),
// potentially an ordering of left-to-right or right-to-left,
// The width of each layer (might do 6 modules in a 8-wide grid),
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
// will either be from a module that gets fixed in the
// current level, to either a module in a prior level,
// or to a module in a propagated bin.
// Make two lists of connections that get computed at each
// level -- and then scan accordingly.
// There's the cost of the modules at this level to the
// outside world, and that gets set.  Then, consider
// each of connections to stuff from the solution tree.
//
// Probably overthinking this -- it should be plenty fast
// enough.  
use pstools;
use rand::Rng;
use crate::{*};

#[derive(Clone)]
pub struct Grid {
    pub x: usize,
    pub y: usize,
    pub bins: Vec<Bin>,
    pub map_mod2bin: Vec<usize>,
    pub edges: Vec<Edge>,
    pub rand_value: u32,  // Used for the scramble, so that results are stable
    pub debug: bool,
} 

#[derive(Copy,Clone)]
pub struct Bin {
    pub x: f32,
    pub y: f32,
    pub grid_x: usize,
    pub grid_y: usize,
    pub index: usize,
}

pub struct Module {
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

// The analytic placer accumulates forces
// pulling/pushing each module in a particular
// direction.  The net force will be divided
// by the weight, and then a module moves a
// certain percentage of distance towards that
// desired location.
#[derive(Copy, Clone)]
struct Force {
    x: f32,
    y: f32,
    weight: f32,
}

pub struct Permlist {
    pub items: Vec<usize>,
}

#[derive(Copy, Clone)]
pub struct ScanLimits {
    pub llx: usize,
    pub lly: usize,
    pub urx: usize,
    pub ury: usize,
}
#[derive(Copy, Clone)]
pub struct ScanWindow {
    pub layer_width: usize, //How many bins in a layer
    pub depth: usize,  // How many layers deep
}

impl fmt::Display for ScanLimits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{},{} - {},{}]:{}", self.llx, self.lly, self.urx, self.ury, (self.urx-self.llx)*(self.ury-self.lly))
    }
}
#[derive(Copy, Clone)]
pub struct ScanStep {
    pub dx: usize,
    pub dy: usize,
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

fn analytic_force(x1: f32, y1: f32, x2: f32, y2: f32, distance: f32) -> (f32, f32, f32, f32, f32) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    if (dx.abs() > distance) || (dy.abs() > distance) {
        return (0.0, 0.0, 0.0, 0.0, 0.0)
    }
    let d = (dx * dx + dy * dy).sqrt();

    if d > distance {
        return (0.0, 0.0, 0.0, 0.0, 0.0)
    }

    let mult = 20.0 * (1.0 - (d/distance));

    let midx = (x1 + x2)/2.0;
    let midy = (y1 + y2)/2.0;

    ( x1 - mult*dx, y1 - mult*dy, x2 + mult*dx, y2 + mult*dy, 1.0)
}
impl Grid {
    pub fn new(x: usize, y: usize)-> Grid {
        let count = (x * y) as usize;
        let mut g = Grid {
            x,
            y,
            bins: Vec::with_capacity(count),
            map_mod2bin: vec!(0 as usize; count),
            edges: Vec::new(),
            rand_value: 8675309,
            debug: false,
        };

        for y in 0..y {
            for x in 0..x {
                let idx = g.grid_index(x, y);
                g.bins.push(Bin { x: x as f32, y: y as f32, grid_x: x, grid_y: y, index: idx });
                g.map_mod2bin[idx] = idx;
            }
        }

        g
    }

    pub fn new_mesh(xdim: usize, ydim: usize) -> Grid {
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
        g
    }

    pub fn save(&self, filepath: String) {
        let mut f = File::create(filepath).unwrap();
        writeln!(&mut f, "{} {}", self.x, self.y).unwrap();
        for b in 0..self.bins.len() {
            writeln!(&mut f, "{} {} {}", self.map_mod2bin[b], self.bins[b].x, self.bins[b].y).unwrap();
        }
        writeln!(&mut f, "{}", self.edges.len()).unwrap();
        for e in &self.edges {
            writeln!(&mut f, "{} {}  {}", e.a, e.b, e.weight).unwrap();
        }
    }

    pub fn load(filepath: String) -> Grid {
        let f = File::open(filepath).unwrap();
        let mut reader = BufReader::new(f);
        let line = getline(&mut reader).unwrap();
        let (dimx, dimy) = scan_fmt!(&line, "{} {}", usize, usize).unwrap();
        println!("Loaded in grid that is {} {}", dimx, dimy);

        let mut g = Grid::new(dimx, dimy);
        for i in 0..g.bins.len() {
            let line = getline(&mut reader).unwrap();
            let (bn, x, y) = scan_fmt!(&line, "{} {} {}", usize, f32, f32).unwrap();
            g.map_mod2bin[i] = bn;
            g.bins[i].x = x;
            g.bins[i].y = y;
        }
        let line = getline(&mut reader).unwrap();
        let ne = scan_fmt!(&line, "{}", usize).unwrap();
        for edge in 0..ne {
            let line = getline(&mut reader).unwrap();
            let (a, b, weight) = scan_fmt!(&line, "{} {} {}", usize, usize, f32).unwrap();
            g.add_edge_idx(a, b, weight);
        }

        g
    }

    pub fn grid_index(&self, x: usize, y: usize) -> usize {
        (y * self.x) + x
    }
    pub fn index_to_grid(&self, n: usize) -> (usize, usize) {
        ((n / self.x), n % self.x)
    }

    // Simple, low quality random number generator -- for the
    // scramble, I don't actually want random numbers -- I want something
    // that is repeatable and predictable, to make debugging easier!
    // Seems like Rust has good (really really random!) random numbers?
    pub fn nextrand(&mut self) -> u32 {
        self.rand_value = (self.rand_value % 77) * (self.rand_value % 33) | (self.rand_value << 8);

        self.rand_value
    }
    
    // Adds an edge to the grid, with a weight.
    pub fn add_edge(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, weight: f32) {
        let idx1 = self.grid_index(x1, y1);
        let idx2 = self.grid_index(x2, y2);
        if (idx1 < idx2) {
            self.edges.push(Edge { a: idx1, b: idx2, weight});
        } else {
            self.edges.push(Edge { a: idx2, b: idx1, weight});
        }
    }

    pub fn add_edge_idx(&mut self, idx1: usize, idx2: usize, weight: f32) {
        if (idx1 < idx2) {
            self.edges.push(Edge { a: idx1, b: idx2, weight});
        } else {
            self.edges.push(Edge { a: idx2, b: idx1, weight});
        }  
    }

    // Computes the weighted cost of all edges, given a mapping of modules to bins
    pub fn grid_cost(&self, map:&Vec<usize>) -> f32 {
        let mut total = 0.0;

        for e in &self.edges {
            // Edge goes from module a to module b -- so get the coords for each, multiply
            // by the weight of the edge
            let ba = map[e.a];
            let bb = map[e.b];
            let x1 = self.bins[ba].x;
            let y1 = self.bins[ba].y;
            let x2 = self.bins[bb].x;
            let y2 = self.bins[bb].y;
            let c = ((x1 - x2).abs() + (y1 - y2).abs()) * e.weight;
            // println!("Edge from module index {}:b{} to {}:b{} == {}", e.a, map[e.a], e.b, map[e.b], c);
            total = total + c;
        }

        total
    }

    pub fn inverse(&self, map: &Vec<usize>) -> Vec<usize> {
        let mut inverse_map = vec!(0 as usize; self.bins.len());
        for i in 0..map.len() {
            inverse_map[map[i]] = i;
        }
        inverse_map
    }
    
    // Pretty-prints the assignment
    pub fn print(&self, map: &Vec<usize>) {
        println!("{} x {} grid", self.x, self.y);
        let inverse_map = self.inverse(map);

        for y in (0..self.y).rev() {
            for x in 0..self.x {
                let bin = self.grid_index(x, y);
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
        
        let inverse_map = self.inverse(&self.map_mod2bin);


        // Get all of the modules within this window of the grid
        let mut scramble_modules = Vec::new();
        for x in llx..urx {
            for y in lly..ury {
                let idx = self.grid_index(x, y);
                scramble_modules.push(inverse_map[idx]);
            }
        }
        // Pull one module out of the scramble at a time, and insert
        // into the grid
        for x in llx..urx {
            for y in lly..ury {
                let idx = self.grid_index(x, y);
                let c = self.pull_random(&mut scramble_modules);
                self.map_mod2bin[c] = idx;
            }
        }        
    }

    pub fn module_net(&self) -> Vec<Vec<usize>> {
        let mut modulenet = Vec::new();
        for i in 0..self.bins.len() {
            modulenet.push(Vec::new());
        }
        for i in 0..self.edges.len() {
            let e = &self.edges[i];
            modulenet[e.a].push(i);
            modulenet[e.b].push(i);
            // println!("Edge {}  connects {} to {}", i, e.a, e.b);
        }

        modulenet
    }

    // opt_layers takes in a series of bin IDs, arranged in layers.
    // Used with the grid code, module maping of modules to bins, and use
    // an inverse map to determine which bin contains a module.
    // From this, we know which modules are mapped to a permutation of
    // bins in a layer.  We can also determine the nets that are
    // in play, and the layer at which a net is closed.
    //
    // Moving code out of the original opt_h, to make it generic, and
    // able to be reused in a bunch of spots.
    pub fn opt_layers(&mut self, layers: Vec<Vec<usize>>, delta: u32) -> (f32, Vec<usize>) {
        if self.debug {
            println!("Optimize {} layers, delta {}", layers.len(), delta);
        }


        let inverse_map = self.inverse(&self.map_mod2bin);

        let mut modulenet = Vec::new();
        for i in 0..self.bins.len() {
            modulenet.push(Vec::new());
        }
        for i in 0..self.edges.len() {
            let e = &self.edges[i];
            modulenet[e.a].push(i);
            modulenet[e.b].push(i);
            if self.debug {
                println!("Edge {}  connects {} to {}  weight {}", i, e.a, e.b, e.weight);
            }
        }

        let mut highmark = vec![0; self.edges.len()];
        // Now determine the last level any particular edge is touched at
        let mut level = 1;
        for layer in &layers {
            if self.debug {
                println!("Checking layer {} with {} elements", level, layer.len());
            }
            for idx in layer {
                if self.debug {
                    println!("MARK grid {} is module {}  inverse (not {})", idx, inverse_map[*idx], self.map_mod2bin[*idx]);
                }
                let module_id = inverse_map[*idx];
                for edge_id in &modulenet[module_id]{
                    highmark[*edge_id] = level;
                    if self.debug {
                        println!("     edge {}  connect {} {}  at level {}", edge_id, self.edges[*edge_id].a, self.edges[*edge_id].b, level);
                    }
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

        if self.debug {
            for i in 0..level {
                println!("Edges at level {}", i);
                for v in &netlevels[i] {
                    println!("Edge {}", v);
                }
            }
        }  

        // HCWT Root Context.  Needed for constructing the HCwT solver,
        // as the type definition needs the generics filled out.  Might
        // be a cleaner way to do this.  The context passed with the HCwT
        // struct doesn't get used here.
        let mut c = Context {
            base_cost: 0.0,
            grid: self.clone(),
        };
        // A shared context that gets passed to generate and filter.
        // This one gets modified by generate, when tracing for the maps
        let mut share_c = Context {
            base_cost: 0.0,
            grid: self.clone(),
        };

        
        // HCWT ROOT NODES
        let root = Node {
            cost: net_cost(&self.map_mod2bin, &netlevels[0], &self.edges, &self.bins),
            selection: 0,
        };
        let mut roots = Vec::new();
        roots.push(root);
        // HCWT ROOT DECISION  (Placeholder)
        let root_dec = Decision {
            modules: Vec::new(),
            mappings: Vec::new(),
            closenets: Vec::new(),
        };

        let mut decs = Vec::new();
        decs.push(root_dec);
        let mut hcwt: HCwT<Context, Decision,Node> = HCwT::new(decs, roots);

        // Now construct the levels        
        level = 1;
        
        for layer in &layers {
            let mut dx = hcwt.new_decision();
            let mut decision = Decision {
                modules: Vec::new(),
                mappings: Vec::new(),
                closenets: netlevels[level].clone(),
            };
            
            // println!("Strip at level: {}", level);
            let mut orig_map = Vec::new();
            for idx in layer {
                // let idx = self.grid_index(x, y);
                decision.modules.push(inverse_map[*idx]);
                orig_map.push(*idx);
                // println!("  Original mapping module {} to bin {}", inverse_map[idx], idx);
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

            level = level + 1;
        }



        // let mut tmpgrid = c;
        hcwt.solve(&mut share_c, &generate, &filter);


        // println!("Solved ----");
        let mut all_nets = Vec::new();
        for i in 0..self.edges.len() {
            all_nets.push(i);
        }

        let last = &hcwt.levels.last().unwrap().nodes;
        let level = hcwt.levels.len() - 1;
        let mut best = net_cost(&self.map_mod2bin, &all_nets, &self.edges, &self.bins);
        let mut best_mapping = self.map_mod2bin.clone();
        let mut mapping = self.map_mod2bin.clone();

        for i in 0..last.len() {
            let node = &last[i];
            // println!("----- SOLUTION {}----", i);

            trace(&hcwt, level, i, &mut mapping);
            // println!("---traced mapping----");
            // self.print(&mapping);

            // let nn_cost = net_cost(&mapping,  &all_nets, &self.edges, &hcwt.user_context.grid.bins);
            let nn_cost = net_cost(&mapping,  &all_nets, &self.edges, &share_c.grid.bins);

            if nn_cost <= best {
                best = nn_cost;
                best_mapping = mapping.clone();
            }
        }

        // println!("Best cost: {}", best);
        
        (best, best_mapping)

    }

    pub fn opt_bi_layers(&mut self, layers: Vec<Vec<usize>>, delta: u32) -> (f32, Vec<usize>) {
        let mut reverse = layers.clone();
        reverse.reverse();

        let mut best_cost = self.grid_cost(&self.map_mod2bin);
        let (cost, map) = self.opt_layers(layers, delta);
        // println!("Forward direction cost: {}", cost);
        self.map_mod2bin = map;
        let (cost, map) = self.opt_layers(reverse, delta);
        //  println!("After reverse, cost {}", cost);

        (cost, map)
    }

    // Strips are vertical stacks, we move horizontally
    pub fn opt_h(&mut self, llx: usize, lly: usize, urx: usize, ury: usize ) -> (f32, Vec<usize>) {
        let mut layers = Vec::new();
        for x in llx..urx {
            let mut layer = Vec::new();
            for y in lly..ury {
                layer.push(self.grid_index(x, y));
            }
            layers.push(layer);
        }

        self.opt_bi_layers(layers, 1000)
    }

    // Strips are horizontal stacks, we move vertically
    pub fn opt_v(&mut self, llx: usize, lly: usize, urx: usize, ury: usize ) -> (f32, Vec<usize>) {
        let mut layers = Vec::new();
        for y in lly..ury {        
            let mut layer = Vec::new();
            for x in llx..urx {
                layer.push(self.grid_index(x, y));
            }
            layers.push(layer);
        }

        self.opt_bi_layers(layers, 100)
    }

    
    pub fn opt_mxn(&mut self, limits: ScanLimits, m: usize, n: usize) -> (f32, Vec<usize>) {
        let mut layers = Vec::new();
        let mut x = limits.llx;
        let mut y = limits.lly;
        let mut best_cost = self.grid_cost(&self.map_mod2bin);
        
        while x < limits.urx
        {
            y = limits.lly;
            while y < limits.ury
            {
                let mut urx = limits.urx.min(x + m);
                let mut ury = limits.ury.min(y + n);

                
                let mut layer = Vec::new();
                for gx in x..urx {
                    for gy in y..ury {
                        layer.push(self.grid_index(gx, gy));
                        // println!("Grid {} {}  -- bin {}", gx, gy, self.grid_index(gx, gy));
                    }
                }
                // println!("mxn layer {} {}   {} {}  with {} bins", x, y, urx, ury, layer.len());
                layers.push(layer);
                y = y + n;
            }
            x = x + m;
        }
        // println!("About to call layer solver on {} layers", layers.len());
        self.opt_bi_layers(layers, 1000)

    }

    pub fn opt_churn_h(&mut self, limits: ScanLimits) -> (f32, Vec<usize>) {
        let mut best_cost = self.grid_cost(&self.map_mod2bin);
        let mut best_mapping = self.map_mod2bin.clone();
        let mut x = limits.llx;
        while x + 2 <= limits.urx {
            let mut l2 = limits;
            l2.llx = x;
            l2.urx = x + 2;
            let (cost, map) = self.opt_mxn(l2, 2, 2);
            // println!("Churn cost {}", cost);
            if cost <= best_cost {
                best_cost = cost;
                best_mapping = map.clone();
                self.map_mod2bin = best_mapping.clone();
            }
            x = x + 1;
        }
        (best_cost, best_mapping)
    }
    pub fn opt_churn_v(&mut self, limits: ScanLimits) -> (f32, Vec<usize>) {
        let mut best_cost = self.grid_cost(&self.map_mod2bin);
        let mut best_mapping = self.map_mod2bin.clone();
        let mut y = limits.llx;
        while y + 2 <= limits.ury {
            let mut l2 = limits;
            l2.lly = y;
            l2.ury = y + 2;
            let (cost, map) = self.opt_mxn(l2, 2, 2);
            // println!("Churn cost {}", cost);
            if cost <= best_cost {
                best_cost = cost;
                best_mapping = map.clone();
                self.map_mod2bin = best_mapping.clone();
            }
            y = y + 1;
        }
        (best_cost, best_mapping)
    }
    

    // Horizontal -- limit the Y axis, can span the space horizontally
    // Vertical -- limit the X axis, can span the space vertically
    pub fn opt_scan(&mut self, limits: ScanLimits, window: ScanWindow, step: ScanStep, horizontal: bool) -> (f32, Vec<usize>) {
        let mut x = limits.llx;
        let mut y = limits.lly;
        let mut best_cost = self.grid_cost(&self.map_mod2bin);
        let mut best_mapping = self.map_mod2bin.clone();
        println!("Scan initial cost: {}", best_cost);
        while x < limits.urx
        {
            y = limits.lly;
            while y < limits.ury
            {
                let mut urx;
                let mut ury;

                if horizontal {
                    urx = limits.urx.min(x + window.depth);
                    ury = limits.ury.min(y + window.layer_width);
                } else {
                    urx = limits.urx.min(x + window.layer_width);
                    ury = limits.ury.min(y + window.depth);
                }


                // println!("Opt window {} {}   {} {} h: {}", x, y, urx, ury, horizontal);
                
                let cost;
                let map;
                if horizontal {
                    (cost, map) = self.opt_h(x, y, urx, ury);
                } else {
                    (cost, map) = self.opt_v(x, y, urx, ury);
                }
                // println!("Cost: {}", cost);
                if cost <= best_cost {
                    best_cost = cost;
                    best_mapping = map.clone();
                }
                self.map_mod2bin = best_mapping.clone();
                y = y + step.dy;
            }
            x = x + step.dx;
        }

        println!("Scan final cost: {}", best_cost);

        (best_cost, best_mapping)
    }

    pub fn analytic_assign(&mut self, limits: ScanLimits, locations: Vec<ModuleAssign>, map_do_we_use: &mut Vec<usize>) {
        // println!("Analytic assign {},  {} entries", limits, locations.len());
        let dx = limits.urx - limits.llx;
        let dy = limits.ury - limits.lly;
        // println!("  DX*DY = {}", dx * dy);

        if dx == 1 && dy == 1 {
            // End case -- assign the remaining module to the bin
            let idx = self.grid_index(limits.llx, limits.lly);
            println!("Assign module {} at {} {} to bin {} {} [{}] ({} {})", locations[0].module, locations[0].x, locations[0].y, limits.llx, limits.lly, idx, self.bins[idx].x, self.bins[idx].y);
            self.map_mod2bin[locations[0].module] = idx;
            return;
        }

        if locations.len() < 1 {
            println!("Why are we here?");
            return;
        }

        let mut sublocations = locations.clone();


        if dy > dx {
            // Split horizontally, sort things by y axis
            sublocations.sort_by_key(|a| (a.y * 100.0) as i32);
            let sp = (limits.ury + limits.lly)/2;
            let mut lower = limits;
            lower.ury = sp;
            let mut higher = limits;
            higher.lly = sp;
            let n = (lower.urx - lower.llx)*(lower.ury - lower.lly);
            // println!("SplitH {} to {} {}  with n=={}", limits, lower, higher, n);
            let mut sl1 = Vec::new();
            let mut sl2 = Vec::new();
            for i in 0..sublocations.len() {
                if i < n {
                    // println!("  Mod {}   {} {} lower", sublocations[i].module, sublocations[i].x, sublocations[i].y);
                    sl1.push(sublocations[i]);
                } else {
                    // println!("  Mod {}   {} {} higher", sublocations[i].module, sublocations[i].x, sublocations[i].y);                    
                    sl2.push(sublocations[i]);
                }
            }
            self.analytic_assign(lower, sl1, map_do_we_use);
            self.analytic_assign( higher, sl2, map_do_we_use);
        } else {
            // Split things vertically, sort things by x axis
            sublocations.sort_by_key(|a| (a.x * 100.0) as i32);
            let sp = (limits.urx + limits.llx)/2;
            let mut left = limits;
            left.urx = sp;
            let mut right = limits;
            right.llx = sp;
            let n = (left.urx - left.llx)*(left.ury - left.lly);
            let mut sl1 = Vec::new();
            let mut sl2 = Vec::new();
            // println!("SplitV {} to {} {}  with n=={}", limits, left, right, n);
            for i in 0..sublocations.len() {
                if i < n {
                    // println!("  Mod {}   {} {} left", sublocations[i].module, sublocations[i].x, sublocations[i].y);
                    sl1.push(sublocations[i]);
                } else {
                    // println!("  Mod {}   {} {} right", sublocations[i].module, sublocations[i].x, sublocations[i].y);
                    sl2.push(sublocations[i]);
                }
            }
            self.analytic_assign(left, sl1, map_do_we_use);
            self.analytic_assign(right, sl2, map_do_we_use);


        }
    }

    pub fn save_analytic(&self, limits: ScanLimits, filename: String) {
        let mut f = File::create(filename).unwrap();
        let d = self.bins[1].x - self.bins[0].x;
        writeln!(&mut f, "{} {} {}", self.bins.len(), self.edges.len(), d).unwrap();
        for x in 0..self.x {
            for y in 0..self.y {
                let idx = self.grid_index(x, y);
                let module = self.map_mod2bin[idx];
                let mut fixed = 0;
                if (x < limits.llx) || (x >= limits.urx) || (y < limits.lly) || (y >= limits.ury) {fixed = 1;}
                writeln!(&mut f, "{} {} {} {} {}", module, self.bins[module].x, self.bins[module].y, 1.0, fixed).unwrap();
            }
        }
        for e in &self.edges {
            writeln!(&mut f, "{} {} {} 0 1", e.a, e.b, e.weight).unwrap();
        }
    }


    pub fn grid_analytic2(&mut self, limits: ScanLimits, scale: f32, alpha: f32, steps: u32, min_distance: f32, steps_d: u32) {
        let mut a = analytic::Analytic::new(self.bins.len());
        // let mut i = 0;
        a.threshold = self.bins[1].x - self.bins[0].x;

        let inverse_map = self.inverse(&self.map_mod2bin);
        for x in 0..self.x {
            for y in 0..self.y {
                let idx = self.grid_index(x, y);
                let module = inverse_map[idx];
                a.nodes[module].tag = module;
                a.nodes[module].location = analytic::Point {
                    x: self.bins[module].x,
                    y: self.bins[module].y,
                };
                a.nodes[module].mass = 1.0;
                let mut fixed = false;
                if (x < limits.llx) || (x >= limits.urx) || (y < limits.lly) || (y >= limits.ury) {fixed = true;}
                a.nodes[module].fixed = fixed;
                // i = i + 1;
            }
        }
        for e in &self.edges {
            a.add_force(e.a, e.b, e.weight, 0.0, true);
        }
        println!("ANALYTIC SOLVER initial cost {}", a.solution_cost());

        a.save("analytic2.ana".to_string());
        a.postscript("initial2.ps".to_string());
        for i in 0..200 {
           a.solver_step(0.3, 0.0);
        }
        println!("ANALYTIC SOLVER force cost {}", a.solution_cost());
        a.postscript("anaforce2.ps".to_string());
        
        // Now add repulsion edges
        for i in 0..a.nodes.len() {
           for j in i+1..a.nodes.len() {
               a.add_force(i, j, 1000.0, a.threshold*0.6, false);
            }
        }
        
        for i in 0..100 {
            a.solver_step(0.2, i as f32 * 0.01);
        }
        a.postscript("anarepulse2.ps".to_string());
        println!("ANALYTIC SOLVER repulse cost {}", a.solution_cost());
        for i in 0..100 {
            a.solver_step(0.4, 1.0);
        }
        
        a.postscript("anarepulse-final2.ps".to_string());
        println!("ANALYTIC SOLVER repulse final cost {}", a.solution_cost());

        let mut modules = Vec::new();
        
        for x in limits.llx..limits.urx {
            for y in limits.lly..limits.ury {
                modules.push(inverse_map[self.grid_index(x, y)]);
            }
        }
        let mut modassign = Vec::new();
        for n in &a.nodes {
            if !n.fixed {
                modassign.push(ModuleAssign {
                    x: n.location.x,
                    y: n.location.y,
                    module: n.tag,
                });
            }
        }
        let mut themap = Vec::new();
        self.analytic_assign(limits, modassign, &mut themap);
        println!("ANALYTIC SOLVER mapped assigned cost {}", a.solution_cost());

        // self.map_mod2bin = themap;
        for node in &mut a.nodes {
            let m = node.tag;
            let b = &self.bins[self.map_mod2bin[m]];
            node.history.truncate(0);
            node.history.push(analytic::Point{x: node.location.x, y: node.location.y});
            node.history.push(analytic::Point{x: b.x, y: b.y});
            node.location.x = b.x;
            node.location.y = b.y;
        }
        println!("ANALYTIC SOLVER snap to place cost {}", a.solution_cost());
        a.print_edge_costs();

        println!("ANALYTIC SOLVER mapped back to grid");
        self.print(&self.map_mod2bin);
        self.print_edge_costs();

        a.postscript("anamapped.ps".to_string());

        
    }

    pub fn print_edge_costs(&self)
    {
        for e in &self.edges {
            let b1 = &self.bins[self.map_mod2bin[e.a]];
            let b2 = &self.bins[self.map_mod2bin[e.b]];

            println!("GE [{}] {} {} -- [{}] {} {} wt {}", e.a, b1.x, b1.y, e.b, b2.x, b2.y, e.weight);
        }
    }

    pub fn grid_analytic(&mut self, limits: ScanLimits, scale: f32, alpha: f32, steps: u32, min_distance: f32, steps_d: u32) {
        // Run through and find the modules that are within this window.
        let inverse_map = self.inverse(&self.map_mod2bin);
        let mut modules = Vec::new();

        for x in limits.llx..limits.urx {
            for y in limits.lly..limits.ury {
                modules.push(inverse_map[self.grid_index(x, y)]);
            }
        }
        println!("{} modules being operated on by analytic engine", modules.len());

        // How many edges go to fixed pads?
        let mut pad_nets = 0;
        for e in &self.edges {
            // FIX ME.  Do we have enough pads?
        }

        // Initialize module locations
        let mut locations = Vec::new();
        for c in 0..self.map_mod2bin.len() {
            let bin = self.map_mod2bin[c];
            let x = self.bins[bin].x * scale;
            let y = self.bins[bin].y * scale;
            let mut pos = Vec::new();
            pos.push((x, y));
            locations.push(ModulePosition{
                    x: x,
                    y: y,
                    history: pos,
            });
        }
        // Initial net centers
        let force = Force {
            x: 0.0,
            y: 0.0,
            weight: 0.0,
        };
        let mut forces = vec![force; self.bins.len()];

        let modulenet = self.module_net();
        for step in 0..steps {
            println!("ITERATION {}", step);
            let mut net_center = Vec::new();
            for i in 0..self.edges.len() {
                let ca = &locations[self.edges[i].a];
                let cb = &locations[self.edges[i].b];
                let cx = (ca.x + cb.x)/2.0;
                let cy = (ca.y + cb.y)/2.0;
                net_center.push(EdgeCenter{
                    x: cx,
                    y: cy,
                });
                // println!("Edge {} links modules {} to {}, center {} {}", i, self.edges[i].a, self.edges[i].b, cx, cy);
            }
            // Compute the forces
            for f in &mut forces {
                (*f).weight = 0.0;
                (*f).x = 0.0;
                (*f).y = 0.0;
            }

            // Pull into relative positons
            if step <= steps_d {
                for module in &modules {
                    let mut x = 0.0;
                    let mut y = 0.0;
                    let mut weight = 0.0;
                    for edge_id in &modulenet[*module] {
                        x = x + net_center[*edge_id].x;
                        y = y + net_center[*edge_id].y;
                        weight = weight + 1.0;
                    }
                
                    forces[*module].x = x;
                    forces[*module].y = y;
                    forces[*module].weight = weight;
                }
            }

            // Use inter-module proximity to push apart
            if step > steps_d {
                println!("Step {} adding repulsive force", step);
                let mut pst = pstools::PSTool::new();
                pst.set_color(0.7, 0.7, 0.7, 1.0);
                let bnd = (self.bins[1].x - self.bins[0].x)/3.0;
                for bin in &self.bins {
                    pst.add_box(bin.x - bnd, bin.y - bnd, bin.x + bnd, bin.y + bnd);
                }
                let bnd2 = bnd / 5.0;
                pst.set_color(0.5, 0.5, 0.8, 1.0);
                for module in &modules {
                    pst.add_box(locations[*module].x - bnd2, locations[*module].y - bnd2, locations[*module].x + bnd2, locations[*module].y + bnd2);
                }
                pst.set_color(1.0, 0.0, 0.0, 1.0);
                for e in &self.edges {
                // Check the distance between two modules -- and if they're too close,
                // add in some weight to push them apart
                    let ax = locations[e.a].x;
                    let ay = locations[e.a].y;
                    let bx = locations[e.b].x;
                    let by = locations[e.b].x;
                    let (dax,day, dbx, dby, w)  = analytic_force(ax, ay, bx, by, min_distance);
                    if w > 0.0 {
                        println!("Mod {} at {} {}, {} at {} {}, A {} {} B {} {}", e.a, ax, ay, e.b, bx, by, dax, day, dbx, dby);
                    }
                    forces[e.a].x = forces[e.a].x + dax;
                    forces[e.a].y = forces[e.a].y + day;
                    forces[e.a].weight = forces[e.a].weight + w;

                    pst.add_line(ax, ay, ax + dax, ay + day);

                    forces[e.b].x = forces[e.b].x + dbx;
                    forces[e.b].y = forces[e.b].y + dby;
                    forces[e.b].weight = forces[e.b].weight + w;

                    pst.add_line(bx, by, bx + dbx, by + dby);
                        
                    }
                    let fp = format!("force{:04}.ps", step);
                    pst.generate(fp);
                }
            
            for module in &modules {
                let weight = forces[*module].weight;
                let mut x = forces[*module].x;
                let mut y = forces[*module].y;

                if weight > 0.0 {
                    x = x / weight;
                    y = y / weight;

                    let dx = x - locations[*module].x;
                    let dy = y - locations[*module].y;
                    let newx = locations[*module].x + alpha * dx;
                    let newy = locations[*module].y + alpha * dy;
                    locations[*module].x = newx;
                    locations[*module].y = newy;
                    locations[*module].history.push((newx,newy));
                    // println!("Move module {} from {} {} to {} {}", module, locations[*module].x, locations[*module].y, newx, newy);                    
                }
            }
        }
        let mut pst = pstools::PSTool::new();
        pst.set_color(0.1, 0.1, 0.1, 1.0);
        for c in &modules {
            pst.add_box(locations[*c].x - 5.0, locations[*c].y - 5.0, locations[*c].x + 5.0, locations[*c].y + 5.0);

        }
        pst.set_color(1.0, 0.0, 0.0, 1.0);
        // println!("FINAL MOTION");
        for module in &modules {
            // println!("module {} pos: {} {}", *module, locations[*module].x, locations[*module].y);
            let mut px = 0.0;
            let mut py = 0.0;
            let mut first = true;
            for pos in &locations[*module].history {
                // println!("  {} {}", pos.0, pos.1);
                if !first {
                    pst.add_line(px, py, pos.0, pos.1);
                }
                px = pos.0;
                py = pos.1;
                first = false;
            }
        }
        pst.generate("analytic.ps".to_string());

        // Now we put the modules into the original bins
        let mut modassign = Vec::new();
        for module in &modules {
            modassign.push(ModuleAssign{
                x: locations[*module].x,
                y: locations[*module].y,
                module: *module,
            });
        }
        let mut themap = Vec::new();
        self.analytic_assign(limits, modassign, &mut themap);
    }


    

}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut final_output = String::new();
        writeln!(f, "{} x {} grid", self.x, self.y);
        let inverse_map = self.inverse(&self.map_mod2bin);

        for y in 0..self.y {
            for x in 0..self.x {
                let bin = self.grid_index(x, y);
                write!(f, "{:3}[{:3}] ", inverse_map[bin], bin);
                //print!("{:3} ", inverse_map[bin]);
            }
            writeln!(f, "");
        }
        writeln!(f, "Total cost: {}", self.grid_cost(&self.map_mod2bin));

        Ok(())
    }
}

pub struct ModulePosition {
    pub x: f32,
    pub y: f32,
    pub history: Vec<(f32,f32)>,
}
pub struct EdgeCenter {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy,Clone)]
pub struct ModuleAssign {
    pub x: f32,
    pub y: f32,
    pub module: usize,
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
        // println!("NCEdge {:3} from module index {}:b{} to {}:b{} == {}", *n, e.a, mapping[e.a], e.b, mapping[e.b], c);
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
            let d = &hcwt.levels[level].options[0];
            for i in 0..d.modules.len() {
                // println!("   module {} maps to bin {}", d.modules[i], d.mappings[node.selection][i]);
                // println!("   module {} maps to bin ..", d.modules[i]);
                //mapping[d.mappings[node.selection][i]] = d.modules[i];
                mapping[d.modules[i]] = d.mappings[node.selection][i];
            }
        //}
    }
}


fn generate(hcwt: &HCwT<Context,Decision,Node>, context: &mut Context, level: usize, decision_index: usize, parent_node_index: usize, nodes: &mut Vec<SelectionNode<Node>>) {
    // println!("Generate for level {}", level);
    let detail = &hcwt.levels[level].options[decision_index];
    let parent_node = &hcwt.levels[level - 1].nodes[parent_node_index];
    
    // Set up the mapping for the prior levels
    // let mut tmpmap = hcwt.user_context.mapping.clone();
    // trace(hcwt, level - 1, parent_node_index, &mut tmpmap);
    trace(hcwt, level - 1, parent_node_index, &mut context.grid.map_mod2bin);

    let d = &hcwt.levels[level].options[0];
    for mi in 0..detail.mappings.len() {
        for i in 0..detail.modules.len() {
            // println!("Generate map module {} to bin {}", detail.modules[i], detail.mappings[mi][i]);
            // tmpmap[d.modules[i]] = d.mappings[mi][i];
            context.grid.map_mod2bin[d.modules[i]] = d.mappings[mi][i];
        }

        let mut new_node = parent_node.detail;
        new_node.selection = mi;
        // let incremental = net_cost(&tmpmap, &d.closenets, &hcwt.user_context.edges, &hcwt.user_context.bins);
        let incremental = net_cost(&context.grid.map_mod2bin, &d.closenets, &context.grid.edges, &context.grid.bins);
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
    /*
    let mut result = nodes.clone();
    result.sort_by_key({|a| a.detail.cost as i32});
    result.truncate(100);

    result
    */
    nodes.sort_by_key({|a| a.detail.cost as i32});
    nodes.truncate(100);
    let result = nodes.clone();

    result
}

struct Context {
    base_cost: f32,
    grid: Grid,
}
struct Decision {
    modules: Vec<usize>,  // modules that are reassigned
    mappings: Vec<Vec<usize>>, // The bins they may go to 
    closenets: Vec<usize>, // Nets that complete at this level
}

#[derive(Copy, Clone)]
struct Node {
    cost: f32,
    selection: usize,
}


pub fn grid_test_hv()
{
    let xdim = 8;
    let ydim = 6;

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
    g.print(&g.map_mod2bin);

    // Scramble the bin assignments
    g.scramble(1, 1, xdim - 1, ydim - 1);
    g.print(&g.map_mod2bin);
    
    let mut limits = ScanLimits {
        llx: 1,
        lly: 1,
        urx: xdim - 1,
        ury: ydim - 1,
        };

    let (c, map) = g.opt_v(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;
    let (c, map) = g.opt_h(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;

    println!("Churn!");
    let (c, map) = g.opt_churn_h(limits);
    g.print(&map);


    let (c, map) = g.opt_v(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;    
    let (c, map) = g.opt_h(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;        
    let (c, map) = g.opt_v(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;    

    println!("Churn!");
    let (c, map) = g.opt_churn_h(limits);
    g.print(&map);


    let (c, map) = g.opt_h(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;            
    let (c, map) = g.opt_v(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;    
    let (c, map) = g.opt_h(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;        
    let (c, map) = g.opt_v(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;    
    let (c, map) = g.opt_h(1, 1, xdim - 1, ydim - 1);
    g.print(&map);
    g.map_mod2bin = map;   

    println!("Churn!");
    let (c, map) = g.opt_churn_h(limits);
    g.print(&map);
}

pub fn grid_test_analytic() {
    let xdim = 18;
    let ydim = 18;

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
    g.print(&g.map_mod2bin);
    // Scramble the bin assignments
    g.scramble(1, 1, xdim - 1, ydim - 1);
    g.print(&g.map_mod2bin);


    let limits = ScanLimits{
        llx: 1,
        lly: 1,
        urx: xdim - 1,
        ury: ydim - 1, 
    };
    g.grid_analytic(limits, 50.0, 0.3, 500, 1000.0, 510);

}

pub fn grid_test_scan() {
    let xdim = 16;
    let ydim = 16;

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
    g.print(&g.map_mod2bin);

    // Scramble the bin assignments
    g.scramble(1, 1, xdim - 1, ydim - 1);
    g.print(&g.map_mod2bin);

    let lim = ScanLimits{
        llx: 1,
        lly: 1,
        urx: xdim - 1,
        ury: ydim - 1,
    };
    let window = ScanWindow{
        layer_width: 4,
        depth: 8,
    };
    let step = ScanStep {
        dx: 2,
        dy: 2,
    };

    for passes in 0..20 {
        println!("Pass {}", passes);
        g.opt_scan(lim, window, step, true);
        g.print(&g.map_mod2bin);

        println!("Churn!");
        g.opt_churn_h(lim);
        g.print(&g.map_mod2bin);

        g.opt_scan(lim, window, step, false);
        g.print(&g.map_mod2bin);

        println!("Churn!");
        g.opt_churn_v(lim);
        g.print(&g.map_mod2bin);
    }  

}

pub fn grid_test () {
    grid_test_scan();
    // grid_test_analytic();

}

use std::io::BufRead;
use std::io::Error;
use std::io::ErrorKind;
fn getline(reader: &mut BufReader<File>) -> std::io::Result<String> {
    loop {
        let mut line = String::new();
        let _len = reader.read_line(&mut line).unwrap();
        // println!("Read in {} bytes, line {}", _len, line);

        if _len == 0 {
            return std::result::Result::Err(Error::new(ErrorKind::Other, "end of file"));
        }

        if line.starts_with("#") {
            // println!("Skip comment.");
            continue;
        }

        if _len == 1 {
            continue;
        }

        return Ok(line.trim().to_string());
    }
    // Error::new(ErrorKind::Other, "Not reachable FILE IO error");
}
