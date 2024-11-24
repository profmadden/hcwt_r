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
