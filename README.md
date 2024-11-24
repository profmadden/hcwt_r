# hcwt_r
Hill Climbing with Trees Rust library

=== 20241120 ===

Have the ferry loading working as an example.  For the
generate function, it might be helpful to have the
ability to update context -- or am I making it all more
complex?

For detail placement, need to compute the costs between
bins (and have to update the locations of bins efficiently).
So....  an array/matrix of positions, and we update that by
tracing back?  Maybe leave the context unmutable, and then
require that the generate function create a mutable instance
of whatever it needs to mutate?  That might be the way
to go.

If the hcwt structure can mutate, then we can't give
references to he decisions and nodes.  So maybe bail on
that.  