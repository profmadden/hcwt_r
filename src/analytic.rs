
// Simple analytic placer
use std::{fmt, io::BufReader};
use scan_fmt::scan_fmt;

use std::fs::File;
use std::io::Write;
use pstools;

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn midpoint(a: &Point, b: &Point) -> Point {
        Point {
            x: (a.x + b.x)*0.5,
            y: (a.y + b.y)*0.5,
        }
    }
}
pub struct Bounds {
    pub llx: f32,
    pub lly: f32,
    pub urx: f32,
    pub ury: f32,
}

#[derive(Clone)]
pub struct Node {
    pub tag: usize,
    pub location: Point,
    pub mass: f32,
    pub history: Vec<Point>,
    pub fixed: bool,
}
pub struct Analytic {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub bounds: Bounds,
    pub debug: bool,
    pub log: bool,
    pub threshold: f32, // Repulsive force between moving objects
}

#[derive(Copy, Clone)]
pub struct Force {
    pub x: f32,
    pub y: f32,
    pub weight: f32,
}

// Attractive forces between nodes
pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub weight: f32,  // How the force acts upon the elements
    pub threshold: f32, // Lower/upper bound
    pub attract: bool,
}

impl Analytic {
    pub fn new(n: usize) -> Analytic {
        let node = Node {
            tag: 0,
            location: Point{x:0.0, y:0.0},
            mass: 0.0,
            history: Vec::new(),
            fixed: false,
        };
        Analytic {
            nodes: vec![node; n],
            edges: Vec::new(),
            bounds: Bounds {
                llx: 0.0,
                lly: 0.0,
                urx: 0.0,
                ury: 0.0,
            },
            debug: false,
            log: false,
            threshold: 0.0,
        }
    }
    pub fn load(filename: String) -> Analytic {
        let f = File::open(filename).unwrap();
        let mut reader = BufReader::new(f);
        let line = getline(&mut reader).unwrap();
        let (num_nodes, num_edges, threshold) = scan_fmt!(&line, "{} {} {}", usize, usize, f32).unwrap();

        let mut a = Analytic::new(num_nodes);
        a.threshold = threshold;
        for i in 0..num_nodes {
            let line = getline(&mut reader).unwrap();
            let (tag, lx, ly, m, fixed) = scan_fmt!(&line, "{} {} {} {} {}", usize, f32, f32, f32, usize).unwrap();
            a.nodes[i].tag = tag;
            a.nodes[i].location = Point{x:lx, y:ly};
            a.nodes[i].mass = m;
            a.nodes[i].fixed = fixed == 1;
        }
        for i in 0..num_edges {
            let line = getline(&mut reader).unwrap();
            let (anode, bnode, w, t, attract) = scan_fmt!(&line, "{} {} {} {} {}", usize, usize, f32, f32, usize).unwrap();
            a.add_force(anode, bnode, w, t, attract == 1);
        }

        a
    }
    pub fn save(&self, filename: String) {
        let mut f = File::create(filename).unwrap();
        writeln!(&mut f, "{} {} {}", self.nodes.len(), self.edges.len(), self.threshold).unwrap();
        for n in &self.nodes {
            let fixed;
            if n.fixed {
                fixed = 1;
            } else {
                fixed = 0;
            }
            writeln!(f, "{} {} {} {} {}", n.tag, n.location.x, n.location.y, n.mass, fixed).unwrap();
        }
        for e in &self.edges {
            let attract;
            if e.attract {
                attract = 1;
            } else {
                attract = 0;
            }
            writeln!(f, "{} {} {} {} {}", e.a, e.b, e.weight, e.threshold, attract).unwrap();
        }
    }

    pub fn solution_cost(&self) -> f32 {
        let mut total = 0.0;
        for e in &self.edges {
            if e.attract {
                let dx = (self.nodes[e.a].location.x - self.nodes[e.b].location.x).abs();
                let dy = (self.nodes[e.a].location.y - self.nodes[e.b].location.y).abs();
                total = total + (dx + dy)*e.weight;
            }
        }

        total
    }

    pub fn print_edge_costs(&self) {
        for e in &self.edges {
            if e.attract {
                let na = &self.nodes[e.a];
                let nb = &self.nodes[e.b];
                println!("AE [{}] {} {} == [{}] {} {} wt {}", e.a, na.location.x, na.location.y, e.b, nb.location.x, nb.location.y, e.weight);
            }
        }      
    }
    pub fn add_force(&mut self, a: usize, b: usize, weight: f32, threshold: f32, attract: bool) {
        self.edges.push(Edge {
            a: a,
            b: b,
            weight: weight,
            threshold: threshold,
            attract: attract,
        });
    }
    pub fn solver_step(&mut self, alpha: f32, rweight: f32) {
        let f = Force {x: 0.0, y: 0.0, weight: 0.0};
        let mut forces = vec![f; self.nodes.len()];

        // Go through each of the edges, and calculate the net force
        for e in &self.edges {
            let p = Point::midpoint(&self.nodes[e.a].location, &self.nodes[e.b].location);
            let dx = self.nodes[e.a].location.x - p.x;
            let dy = self.nodes[e.a].location.y - p.y;

            let mut fm = (dx * dx + dy * dy).sqrt();
            if e.attract == false {
                if fm < e.threshold {
                    // println!("Distance {}, will repulse", fm);
                    fm = fm - e.threshold;
                    fm = fm * rweight;
                } else {
                    fm = 0.0;
                }
            } else {
                fm = fm * (1.0 - rweight);
            }

            forces[e.a].x = forces[e.a].x - dx * fm * e.weight;
            forces[e.a].y = forces[e.a].y - dy * fm * e.weight;      
            forces[e.a].weight = forces[e.a].weight + e.weight.abs() * fm.abs() ;

            let dx = self.nodes[e.b].location.x - p.x;
            let dy = self.nodes[e.b].location.y - p.y;
            forces[e.b].x = forces[e.b].x - dx * fm * e.weight;
            forces[e.b].y = forces[e.b].y - dy * fm * e.weight;      
            forces[e.b].weight = forces[e.b].weight + e.weight.abs() * fm.abs();
        }

        // Now apply the forces to each of the nodes
        for i in 0..self.nodes.len() {
            if forces[i].weight > 0.0 {
                forces[i].x = forces[i].x / forces[i].weight;
                forces[i].y = forces[i].y / forces[i].weight;
            }
            if !self.nodes[i].fixed {
                let loc = self.nodes[i].location;
                self.nodes[i].history.push(loc);
                self.nodes[i].location.x = self.nodes[i].location.x + alpha * forces[i].x;
                self.nodes[i].location.y = self.nodes[i].location.y + alpha * forces[i].y;
            }
        }
    }
    pub fn postscript(&self, filename: String) {
        let mut pst = pstools::PSTool::new();
        let mut dim = 1.0;
        if self.threshold != 0.0 {
            pst.set_scale(200.0/self.threshold); 
            dim = self.threshold * 0.2;
        }

        for i in 0..self.nodes.len() {
            if self.nodes[i].fixed {
                pst.set_fill(false);
                pst.set_color(0.0, 0.0, 0.0, 1.0);
            } else {
                pst.set_fill(true);
                pst.set_color(0.2, 0.2, 0.2, 1.0);
            }

            pst.add_box(self.nodes[i].location.x - dim, self.nodes[i].location.y - dim, self.nodes[i].location.x + dim, self.nodes[i].location.y + dim);
        }
        pst.set_color(1.0, 0.0, 0.0, 1.0);
        for i in 0..self.nodes.len() {
            if self.nodes[i].history.len() > 1 {
                let h = &self.nodes[i].history;
                for j in 1..self.nodes[i].history.len() {
                    pst.add_line(h[j - 1].x, h[j - 1].y, h[j].x, h[j].y);
                }
            }
        }
        pst.set_color(1.0, 0.3, 0.3, 1.0);
        for i in 0..self.nodes.len() {
            pst.add_text(self.nodes[i].location.x, self.nodes[i].location.y, format!("{}", self.nodes[i].tag));
        }
        pst.generate(filename);
    }
}


pub fn analytic_test() {
    let num_nodes = 5;

    let mut a = Analytic::new(num_nodes);
    a.nodes[0].fixed = true;
    a.nodes[0].location.x = 0.0;
    a.nodes[0].location.y = 0.0;

    a.nodes[1].fixed = true;
    a.nodes[1].location.x = 200.0;
    a.nodes[1].location.y = 200.0;


    a.nodes[2].fixed = false;
    a.nodes[2].location.x = 146.0;
    a.nodes[2].location.y = 200.0;


    a.nodes[3].fixed = false;
    a.nodes[3].location.x = 10.0;
    a.nodes[3].location.y = 20.0;

    a.nodes[4].fixed = false;
    a.nodes[4].location.x = 55.0;
    a.nodes[4].location.y = 10.0;

    a.add_force(0, 2, 1.0, 0.0, true);
    a.add_force(1, 2, 2.2, 0.0, true);
    a.add_force(0, 3, 1.0, 0.0, true);
    a.add_force(1, 3, 1.0, 0.0, true);
    a.add_force(2, 3, 30.0, 20.0, false);
    
    a.add_force(0, 4, 1.0, 0.0, true);
    a.add_force(3, 4, 1.0, 0.0, true);
    
    a.postscript("initial.ps".to_string());
    for i in 0..200 {
        a.solver_step(0.1, 0.0);
    }
    a.postscript("analytic.ps".to_string());


}

pub fn analytic_test_file(filename: String) {
    let mut a = Analytic::load(filename);

      
    a.postscript("initial.ps".to_string());
    for i in 0..200 {
        a.solver_step(0.1, 0.0);
    }
    a.postscript("analytic.ps".to_string());
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
