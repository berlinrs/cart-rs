#[macro_use]
extern crate quickcheck;
extern crate cart;
extern crate rand;

use self::quickcheck::{Arbitrary, Gen};

// The maximum key size. keeping it relatively
// small increases the chance of multiple
// operations being executed against the same
// key, which will tease out more bugs.
const KEY_SPACE: u8 = 20;

#[derive(Clone, Debug)]
enum Op {
    Set(u8, u8),
    Get(u8),
}
use Op::{Get, Set};

// Arbitrary lets you create randomized instances
// of types that you're interested in testing
// properties with. QuickCheck will look for
// this trait for things that are the arguments
// to properties that it is testing.
impl Arbitrary for Op {
    fn arbitrary<G: Gen>(g: &mut G) -> Op {
        // pick a random key to perform an operation on
        let k: u8 = g.gen_range(0, KEY_SPACE);

        if g.gen_weighted_bool(2) {
            Set(k, g.gen())
        } else {
            Get(k)
        }
    }
}

fn prop_impl_matches_model(ops: Vec<Op>) -> bool {
    let mut implementation = cart::Art::default();
    let mut model = std::collections::BTreeMap::new();

    for op in ops {
        match op {
            Set(k, v) => {
                implementation.set(vec![k; k as usize], v);
                model.insert(k, v);
            }
            Get(k) => {
                if implementation.get(&*vec![k; k as usize]) != model.get(&k) {
                    return false;
                }
            }
        }
    }

    true
}

// This macro is shorthand for creating a test
// function that calls the property functions inside.
// QuickCheck will generate a Vec of Op's of default
// length 100, which can be overridden by setting the
// QUICKCHECK_GENERATOR_SIZE env var or creating your
// own type that implements Arbitrary and using it as
// an argument to the property function.
quickcheck! {
    fn implementation_matches_model(ops: Vec<Op>) -> bool {
        prop_impl_matches_model(ops)
    }
}

#[test]
fn test_1() {
    // postmortem 1: were not properly handling prefix mismatches
    prop_impl_matches_model(vec![
        Set(15, 67),
        Set(9, 182),
        Set(12, 221),
        Set(16, 122),
        Set(3, 41),
        Set(5, 209),
        Set(2, 96),
        Set(10, 227),
        Set(13, 37),
        Set(4, 182),
        Set(17, 218),
        Set(6, 139),
        Set(18, 249),
        Set(19, 209),
        Set(14, 34),
        Set(11, 104),
        Set(8, 89),
        Set(1, 110),
    ]);
}

#[test]
fn test_2() {
    // postmortem 1:
    prop_impl_matches_model(vec![
        Set(9, 58),
        Set(4, 10),
        Set(2, 209),
        Set(5, 3),
        Set(14, 175),
        Set(1, 73),
        Set(8, 53),
        Set(18, 244),
        Set(12, 227),
        Set(15, 255),
        Set(3, 92),
        Set(6, 102),
        Set(19, 239),
        Set(17, 240),
        Set(7, 227),
        Set(11, 41),
        Set(16, 15),
        Set(10, 215),
        Set(10, 82),
    ]);
}

#[test]
fn test_3() {
    // postmortem 1:
    prop_impl_matches_model(vec![]);
}
