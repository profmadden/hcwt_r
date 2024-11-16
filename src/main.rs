use pstools_r;
// mod lib;

fn main() {
    println!("Hello, world!  Hill climbing with trees demo");
    println!("PSTools version {}", pstools_r::psversion());
    // crate::lib::hcwt_test();
    hcwt_r::hcwt_test();
}
