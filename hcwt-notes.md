### 20241122

Basic HCwT engine is running, able to trace back parent
nodes to find how each individual configuration is constructed.
The main program has a ferry loading demo, and that works
nicely -- doing a Pareto front with just two variables is
very easy.  I'll to add multi-dimensional Pareto to the
code (and make it struct-generic?).

Working on a grid interface as well.  Given an X by Y
grid, with potentially an internal window to operate
on, and then a list of edges -- maybe just do point-to-point
for the initial implementation.  There are X*Y "cells"
that can be shuffled, and X*Y mappings of cells to bins.

The grid HCwT should be able to scan in the X or Y
direction, and have strips of whatever width and height.
We build an HCwT decision for each strip, enumerate the
different bin mappings, and then throw it to the solver.
Output is the revised mapping.

If I've got 100x100 grid, that's only 10000 different
possible edge pairs -- so a limited number of edges.
Can clump nets together and weight accordingly, and
even limit things to a certain percentage of the total
weight (likely only a tiny fraction of the edges will
matter).  

Every option for a HCwT decision will have an array
of locations for a specific permutation (and a bin
ID number).  The generate can trace back, and knowing
the option selected, determine the position of each
moving cell.  We map all the positions, then compute
the total cost of nets that close consideration
on the level.  

----
Working on the grid.  Have a simple test case going,
where I can generate a grid of arbitrary size, and
add mesh connections.  HPWL calculations look OK.

Will build the HCwT solver for it tomorrow.

### 20241123

Have the grid HCwT mostly running.  Need to make sure
the nodes and decision levels sync up -- might have
some sort of off-by-one issue?  The root node is
on a level without any decision, so the index of
the decision should be off by 1 from the index of
the solution tree layers.  Got to be a cleaner way
to do that.  Maybe have an empty decision level,
that gets inserted?  Ghost level?  Then the tree
level would match exactly the decision level, and
level zero would be a special case.

Yeah, that's probably the way to go.  More grief
otherwise.

----

Hack under way.  Will merge the nodes within the
decision struct, and get rid of the separate solution
tree.  No need for the HCwT Layer.  And should change
to Hcwt, maybe?

Each decision may have an implicit choice (ferry loading),
or a set of explicity choices (a vector of options).   Which
way you go depends on what makes sense for the particular
problem.

For things like patient scheduling -- we can have one
HWcT generate assignments of patients to rooms, and
then feed the selected options from that as roots for
a different HCwT solver that focuses on assignment of
nurses.  After that, the resulting selected options
get fed to an equipment allocator.  Or something like
that.

### 20241125

Got the incremental costs working, which lets me
prune the solution space.  In a 8x5 grid, where there
are three cells stacked vertically, in 6 strips,
thats 6^6 combinations, 46,656.  The best rearrangement
goes down to 183.  With a limit of 100 live branches,
I get the same result.  This pruning works.  Big
window, lot of freedom of movement, and we get the
good answers in the blink of an eye.

Need to clean up the code, put the upper limit for
pruning into the context, and figure out how to
sort on floating point number.  Ordered Float, right?

### 20241126

Woot.  Have the generic optimization of layers
(handing in a vector of vectors of bin numbers),
and I'm getting the same results with the horizontal
reshuffle.  Should be a trivial hack to change this
to vertical strips, and then a window-in-window scan.

Rust is pretty freakin' cool.

### 20241127

Got the scanner going -- but bottoming out pretty
quickly.  Need to have a decent starting point if there's
going to be progress.  If everything is random jumble,
then everything has pretty much no gain going in any
particular direction.

Will build grid_analytic, that takes a window, and then
runs a simple analytic engine.  Each net has a center
of mass (the halfway point), and then each cell has
a target destination.  Move the cell towards the target by
some epsilon amount.

Keep doing that, then snap things in place by sorting by
X into layers, and then assigning back to the grid.

Any net that touches a fixed resource gets double weight,
or something like that.

Right now, on an 8x8 grid scramble, optimal is 112, scramble
gets 390, and then two scans in alternate directions
brings it down to 328.

----
If we start with a random placement, only small local
changes are going to do anything.  Have to have a not
totally trash start, and then maybe we're not stuck
on a plateau, but instead have an incline we can
stumble down (with large window optimization getting
us over small bumps).

Add a grid rotate?  Groups of 4, have things be able
to turn?  Or all 24 permutations of the space.

The challenge with placement is that the optimal location
of any cell depends on the locations of other cells
that are connected to it.  If the other cells are
in bad locations, the cell in question is stuck... in
a bad location.  Lots of stable, low quality local
minima, and placers can get wedged into them.  The trick
is to get the majority of cells into good locations,
so that we're on the side of a slope that descends into
a good local optima (and not a bad local optima).
Being able to get over small bumps in the cost surface
aids in getting to the best local optima in a valley.

With the big scramble, seem to be stuck in a bad
local optima.  Will try the 2x2 layering, see what happens.

---
Should also have a single-layer, where we try all permutations
of a group of bins.  Brute force enumeration, but could
do a 3x3 window, with 9! = 362,880.  Expensive, but might
get out of a local minima.  The optimizations are in some
sense a jump out of a local minima to another spot in
the cost surface.  The 2x2 is only 24 combinations.

Instead of 2x2 specifically, maybe allow permutations of
m x n?  Then do a strip m or n wide, stacking them up
in layers.  All permutations in that m x n level, and
then the next layer does the HCwT pruning.

Also should add a postscript generator, to show
how cells have moved between start and finish.
Pack trail sort of thing.  Good time to work on
pstools_r.

----

Analytic seems to be doing the right thing on grids;
quickly converges to uniformly spaced cells from the
mesh.  Niiiiice.  Also have integration with the
PostScript generator, and the plots show the motion
of the cells across iterations.  Might want to break
things out to a separate analytic engine.

Need to add some code to convert the analytic placement
results back into bin assignments.  I'll have a
rectangle of bins to assign to, and all the cells with
XY coords.  Split things recursively, in either the
H or V direction, and then assign some of the cells to
one side or the other, recursively.  Recursion stops
when there's only one cell, and one bin.

I should pull in the old IBM01 standard cell benchmark,
and see how things go.  The macro blocks make stuff
harder, so push that off.

Integration with fsdc_r, that's on the plate for today.

Legalization/bin assignment could also be done with
dual row.  But fight that battle another day.

### 20241128

Renamed cells to modules -- so the grid operates
on modules.  Should make the code a little more
clear and less debug-challenging when integrating
with the standard cell/mixed-size placer.

### 20241130

Have the window for scans modified to use
layer_width and depth, and then opt_scan uses
the appropriate dimensions (avoid cases where
I generate a zillion different permutations
on a wide level).  Also have basic churn running.
On a 16x16 scramble, optimal cost is 480, and
a simple run with multiple passes gets down
to 856 from an initial 4074.  So much of the
way there.  Still hit a local optima, but not
too damn shabby.

Compare against conventional small window -- should
be no contest.  Maybe have a paper focusing on the
churn effect for HCwT, and the run time impact.
Focus on FPGAs?

With more iterations, down to 760.  Damn.  It
actually freakin' works.

The analytic run should get us in the right
ballpark, and then HCwT can dial it in.  Cross-bin
repartitioning cleans up the clusters.  This could
get a big win over the other tools.  Need to build
the SHARP, and then go after ICCAD 04 IBM05.

Might want to generate a PDF that shows distance
from optimal (assuming it's a mesh).  Might give an
idea of what sort of transformations might be
effective.

### 20241205

Started gridopt -- stand alone binary to load in 
a grid, and then perform optimization.  There's a
grid that will come in from the SHARP k-way cluster
generation, need to work on the analytic engine and
get that working nicely.

Also, splitting things out into a separate analytic.rs,
that should be stand-alone.  Grid will use that as
a library, rather than something internal to the grid.
Should be more useful in other applications.

