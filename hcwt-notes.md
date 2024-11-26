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
