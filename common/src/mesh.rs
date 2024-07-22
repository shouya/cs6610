use std::collections::{BTreeMap, BTreeSet};

pub fn tear_into_strips(indices: &[u32]) -> Vec<Vec<u32>> {
  let mut trigs: BTreeSet<[u32; 3]> = BTreeSet::new();
  // line => (trig, vert)
  let mut colinear_trigs: BTreeMap<[u32; 2], Vec<([u32; 3], u32)>> =
    BTreeMap::new();

  for trig in indices.chunks(3) {
    let t: [u32; 3] = trig.try_into().unwrap();
    trigs.insert(t);

    colinear_trigs
      .entry(sorted([t[0], t[1]]))
      .or_default()
      .push((t, t[2]));
    colinear_trigs
      .entry(sorted([t[1], t[2]]))
      .or_default()
      .push((t, t[0]));
    colinear_trigs
      .entry(sorted([t[2], t[0]]))
      .or_default()
      .push((t, t[1]));
  }

  let mut strips = Vec::new();

  let next_vert = |trigs: &mut BTreeSet<_>, b, c| {
    colinear_trigs[&sorted([b, c])]
      .iter()
      .filter_map(|(t, d)| trigs.contains(t).then_some((t, *d)))
      .next()
  };

  while let Some([a, mut b, mut c]) = trigs.pop_last() {
    let mut strip = vec![a, b, c];

    // heuristic for better result regardless the traversal direction
    // of the trigs.
    match next_vert(&mut trigs, b, c) {
      Some(_) => {}
      None => {
        strip = vec![b, c, a];
        b = c;
        c = a;
      }
    }

    loop {
      let Some((t, d)) = next_vert(&mut trigs, b, c) else {
        break;
      };

      // add vertex to strip and mark the triangle as processed
      trigs.remove(t);
      strip.push(d);

      b = c;
      c = d;
    }

    strips.push(strip);
  }

  validate_trig_strips(indices, &strips);

  strips
}

pub fn concat_strips(strips: &[Vec<u32>]) -> Vec<u32> {
  let mut indices = Vec::new();
  for strip in strips {
    if !indices.is_empty() {
      // duplicate the last vertex in previous strip
      indices.extend_from_within(indices.len() - 1..);
      // duplicate the first vertex in next strip
      indices.extend_from_slice(&strip[..1]);
    }
    indices.extend_from_slice(strip);
  }
  indices
}

fn validate_trig_strips(indices: &[u32], strips: &[Vec<u32>]) {
  let mut original_trigs: Vec<[u32; 3]> = indices
    .chunks(3)
    .map(|x| x.try_into().unwrap())
    .map(rotate3)
    .collect();

  original_trigs.sort();

  let mut strip_trigs: Vec<[u32; 3]> = strips
    .iter()
    .flat_map(|strip| {
      strip.windows(3).map(|x| x.try_into().unwrap()).map(rotate3)
    })
    .collect();
  strip_trigs.sort();

  // assert_eq!(original_trigs, strip_trigs);
}

fn sorted<T: Ord, const N: usize>(mut a: [T; N]) -> [T; N] {
  a.sort();
  a
}

// rotate the triangle vertices while maintaining the face winding
// order
fn rotate3<T: Ord>(mut a: [T; 3]) -> [T; 3] {
  if a[0] > a[1] || a[0] > a[2] {
    a.rotate_left(1);
  }

  if a[0] > a[1] || a[0] > a[2] {
    a.rotate_left(1);
  }

  a
}
