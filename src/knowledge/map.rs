//! The knowledge map (change 171): which entries are near in meaning, how they
//! group, and where each one sits in 2D.
//!
//! Modelled on `~/dev/buddypia/personal-knowledge`'s atlas, which projects
//! nothing: near neighbours become springs, Louvain finds the groups, and a
//! force simulation places the points. This one is deterministic and keeps its
//! positions, so the same knowledge stays where it was. Everything here is a
//! pure function; the database read and write are in `knowledge.rs`, the
//! drawing in `ui/knowledge_map.rs`.

use crate::*;
use std::collections::BTreeMap;

/// One group: its name, its members by centrality, and its outline.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MapGroup {
    pub(crate) name: String,
    /// Indices into the map's points, most central first.
    pub(crate) members: Vec<usize>,
    /// Convex hull of the members' positions, counter-clockwise.
    pub(crate) hull: Vec<[f32; 2]>,
    pub(crate) freshness: f64,
}

/// A finished map. Its points are in the order of the entries it was computed
/// from, so it is only ever drawn against that snapshot.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct KnowledgeMap {
    pub(crate) entry_ids: Vec<i64>,
    pub(crate) fingerprints: Vec<Option<u64>>,
    pub(crate) points: Vec<[f32; 2]>,
    pub(crate) group_of: Vec<Option<usize>>,
    /// Pairs of point indices, lower first, each once.
    pub(crate) edges: Vec<(usize, usize)>,
    /// The other end of every edge, per point.
    pub(crate) linked: Vec<Vec<usize>>,
    pub(crate) groups: Vec<MapGroup>,
    /// The order entry titles are tried in when labels are placed.
    pub(crate) label_order: Vec<usize>,
    pub(crate) min: [f32; 2],
    pub(crate) max: [f32; 2],
    /// What the layout did, for the tests and the logs.
    pub(crate) placement: MapPlacement,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MapPlacement {
    #[default]
    Fresh,
    Settled,
    Reused,
}

/// A map, and the positions to save when the layout ran.
#[derive(Debug, Clone)]
pub(crate) struct MapOutcome {
    pub(crate) map: KnowledgeMap,
    pub(crate) save: Option<Vec<SavedPosition>>,
}

/// 64-bit FNV-1a over the vector's bytes: which vector a position was placed by.
pub(crate) fn vector_hash(vector: &[f32]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for value in vector {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

/// Dot product in eight lanes, so the compiler can vectorise it; a plain
/// `sum()` of products is a serial chain it may not reorder.
fn dot(left: &[f32], right: &[f32]) -> f32 {
    let mut lanes = [0.0_f32; 8];
    let left_chunks = left.chunks_exact(8);
    let right_chunks = right.chunks_exact(8);
    let tail: f32 = left_chunks
        .remainder()
        .iter()
        .zip(right_chunks.remainder())
        .map(|(a, b)| a * b)
        .sum();
    for (a, b) in left_chunks.zip(right_chunks) {
        for lane in 0..8 {
            lanes[lane] += a[lane] * b[lane];
        }
    }
    lanes.iter().sum::<f32>() + tail
}

/// Each entry's nearest others by cosine, most similar first, at most `count`.
/// An entry without a vector has none and is nobody's neighbour.
pub(crate) fn map_neighbours(vectors: &[Option<Vec<f32>>], count: usize) -> Vec<Vec<(usize, f32)>> {
    let n = vectors.len();
    let mut lists: Vec<Vec<(usize, f32)>> = vec![Vec::with_capacity(count + 1); n];
    if count == 0 {
        return lists;
    }
    for i in 0..n {
        let Some(left) = &vectors[i] else { continue };
        for j in (i + 1)..n {
            let Some(right) = &vectors[j] else { continue };
            let similarity = dot(left, right);
            keep_nearest(&mut lists[i], (j, similarity), count);
            keep_nearest(&mut lists[j], (i, similarity), count);
        }
    }
    lists
}

/// Insert keeping the list sorted by similarity, descending; an equal
/// similarity keeps the earlier index first, so the order is deterministic.
fn keep_nearest(list: &mut Vec<(usize, f32)>, candidate: (usize, f32), count: usize) {
    if list.len() == count && candidate.1 <= list[count - 1].1 {
        return;
    }
    let at = list
        .iter()
        .position(|(_, similarity)| candidate.1 > *similarity)
        .unwrap_or(list.len());
    list.insert(at, candidate);
    list.truncate(count);
}

/// The lines drawn: an entry's `KNOWLEDGE_MAP_EDGES_PER_ENTRY` nearest that are
/// at least `KNOWLEDGE_MAP_EDGE_MIN_COSINE` alike.
pub(crate) fn map_edges(neighbours: &[Vec<(usize, f32)>]) -> Vec<(usize, usize)> {
    let mut edges: Vec<(usize, usize)> = neighbours
        .iter()
        .enumerate()
        .flat_map(|(i, list)| {
            list.iter()
                .take(KNOWLEDGE_MAP_EDGES_PER_ENTRY)
                .filter(|(_, similarity)| *similarity >= KNOWLEDGE_MAP_EDGE_MIN_COSINE)
                .map(move |(j, _)| (i.min(*j), i.max(*j)))
        })
        .collect();
    edges.sort_unstable();
    edges.dedup();
    edges
}

/// The neighbour graph as symmetric weights `cosine − baseline`, positive only,
/// each row sorted by index.
fn weighted_graph(neighbours: &[Vec<(usize, f32)>]) -> Vec<Vec<(usize, f64)>> {
    let mut rows: Vec<BTreeMap<usize, f64>> = vec![BTreeMap::new(); neighbours.len()];
    for (i, list) in neighbours.iter().enumerate() {
        for (j, similarity) in list {
            let weight = f64::from(*similarity - KNOWLEDGE_MAP_GROUP_BASELINE);
            if weight <= 0.0 {
                continue;
            }
            for (from, to) in [(i, *j), (*j, i)] {
                let slot = rows[from].entry(to).or_insert(0.0);
                *slot = slot.max(weight);
            }
        }
    }
    rows.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

/// Louvain modularity communities: local moves in index order until none
/// improves, then the communities become nodes, until nothing moves. No
/// randomness, and a tie keeps the lower community, so the same graph always
/// gives the same answer.
fn louvain(graph: &[Vec<(usize, f64)>]) -> Vec<usize> {
    let mut of_node: Vec<usize> = (0..graph.len()).collect();
    let mut level: Vec<Vec<(usize, f64)>> = graph.to_vec();
    // Each level merges at least two nodes or stops, so this is bounded by n.
    for _ in 0..graph.len().max(1) {
        let (assignment, moved) = louvain_level(&level);
        let (assignment, count) = renumbered(&assignment);
        for community in &mut of_node {
            *community = assignment[*community];
        }
        if !moved || count == level.len() {
            break;
        }
        level = aggregated(&level, &assignment, count);
    }
    of_node
}

fn louvain_level(graph: &[Vec<(usize, f64)>]) -> (Vec<usize>, bool) {
    let n = graph.len();
    let degree: Vec<f64> = graph
        .iter()
        .map(|row| row.iter().map(|(_, weight)| weight).sum())
        .collect();
    let two_m: f64 = degree.iter().sum();
    let mut community: Vec<usize> = (0..n).collect();
    if two_m <= 0.0 {
        return (community, false);
    }
    let mut total = degree.clone();
    let mut moved_any = false;
    let mut links: Vec<(usize, f64)> = Vec::new();
    // Each pass either moves a node to a strictly better community or stops;
    // the cap only guards against floating-point cycling.
    for _ in 0..(4 * n).max(16) {
        let mut moved = false;
        for node in 0..n {
            let own = community[node];
            links.clear();
            for (other, weight) in &graph[node] {
                if *other == node {
                    continue;
                }
                let target = community[*other];
                match links.iter_mut().find(|(c, _)| *c == target) {
                    Some(link) => link.1 += weight,
                    None => links.push((target, *weight)),
                }
            }
            links.sort_unstable_by_key(|(c, _)| *c);
            total[own] -= degree[node];
            let gain = |community: usize, weight: f64, total: &[f64]| {
                weight - total[community] * degree[node] / two_m
            };
            let own_weight = links
                .iter()
                .find(|(c, _)| *c == own)
                .map_or(0.0, |(_, weight)| *weight);
            let mut best = own;
            let mut best_gain = gain(own, own_weight, &total);
            for (candidate, weight) in &links {
                let candidate_gain = gain(*candidate, *weight, &total);
                if candidate_gain > best_gain + 1e-12 {
                    best = *candidate;
                    best_gain = candidate_gain;
                }
            }
            total[best] += degree[node];
            if best != own {
                community[node] = best;
                moved = true;
                moved_any = true;
            }
        }
        if !moved {
            break;
        }
    }
    (community, moved_any)
}

/// Communities numbered 0.. in order of first appearance.
fn renumbered(assignment: &[usize]) -> (Vec<usize>, usize) {
    let mut map: BTreeMap<usize, usize> = BTreeMap::new();
    let mut next = 0;
    let numbered = assignment
        .iter()
        .map(|community| {
            *map.entry(*community).or_insert_with(|| {
                next += 1;
                next - 1
            })
        })
        .collect();
    (numbered, next)
}

fn aggregated(
    graph: &[Vec<(usize, f64)>],
    assignment: &[usize],
    count: usize,
) -> Vec<Vec<(usize, f64)>> {
    let mut rows: Vec<BTreeMap<usize, f64>> = vec![BTreeMap::new(); count];
    for (node, row) in graph.iter().enumerate() {
        for (other, weight) in row {
            *rows[assignment[node]]
                .entry(assignment[*other])
                .or_insert(0.0) += weight;
        }
    }
    rows.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

/// A group index per entry, or none: communities of at least
/// `KNOWLEDGE_MAP_MIN_GROUP`, numbered largest first (ties: lowest member).
pub(crate) fn map_groups(neighbours: &[Vec<(usize, f32)>]) -> Vec<Option<usize>> {
    let communities = louvain(&weighted_graph(neighbours));
    let mut members: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (node, community) in communities.iter().enumerate() {
        members.entry(*community).or_default().push(node);
    }
    let mut kept: Vec<Vec<usize>> = members
        .into_values()
        .filter(|nodes| nodes.len() >= KNOWLEDGE_MAP_MIN_GROUP)
        .collect();
    kept.sort_by(|left, right| right.len().cmp(&left.len()).then(left[0].cmp(&right[0])));
    let mut group_of = vec![None; neighbours.len()];
    for (group, nodes) in kept.iter().enumerate() {
        for node in nodes {
            group_of[*node] = Some(group);
        }
    }
    group_of
}

/// Each group's members, most central first: centrality is the summed weight to
/// the other members; a tie goes to the lower entry id.
fn central_members(
    group_of: &[Option<usize>],
    neighbours: &[Vec<(usize, f32)>],
    entry_ids: &[i64],
) -> Vec<Vec<usize>> {
    let graph = weighted_graph(neighbours);
    let count = group_of.iter().flatten().max().map_or(0, |last| last + 1);
    let mut groups: Vec<Vec<(usize, f64)>> = vec![Vec::new(); count];
    for (node, group) in group_of.iter().enumerate() {
        let Some(group) = group else { continue };
        let centrality = graph[node]
            .iter()
            .filter(|(other, _)| group_of[*other] == Some(*group))
            .map(|(_, weight)| weight)
            .sum();
        groups[*group].push((node, centrality));
    }
    groups
        .into_iter()
        .map(|mut members| {
            members.sort_by(|left, right| {
                right
                    .1
                    .total_cmp(&left.1)
                    .then(entry_ids[left.0].cmp(&entry_ids[right.0]))
            });
            members.into_iter().map(|(node, _)| node).collect()
        })
        .collect()
}

/// A group's name: its most central entry's title, shortened.
pub(crate) fn map_group_name(title: &str) -> String {
    let mut characters = title.chars();
    let head: String = characters
        .by_ref()
        .take(KNOWLEDGE_MAP_GROUP_LABEL_CHARS)
        .collect();
    if characters.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

/// The two leading principal directions of the centred vectors, by power
/// iteration from fixed starts (the second kept orthogonal to the first), each
/// signed so its largest component is positive. Returns each vector's
/// coordinates on them.
pub(crate) fn principal_plane(vectors: &[&[f32]]) -> Vec<[f32; 2]> {
    let n = vectors.len();
    if n == 0 {
        return Vec::new();
    }
    let dimensions = vectors[0].len();
    let mut mean = vec![0.0_f32; dimensions];
    for vector in vectors {
        for (sum, value) in mean.iter_mut().zip(vector.iter()) {
            *sum += value / n as f32;
        }
    }
    let centred: Vec<Vec<f32>> = vectors
        .iter()
        .map(|vector| vector.iter().zip(&mean).map(|(v, m)| v - m).collect())
        .collect();
    let mut axes: Vec<Vec<f32>> = Vec::with_capacity(2);
    for axis in 0..2 {
        let mut direction: Vec<f32> = (0..dimensions)
            .map(|d| ((d * (7 + axis * 6) + 3) % 17) as f32 - 8.0)
            .collect();
        for _ in 0..60 {
            let mut next = vec![0.0_f32; dimensions];
            for row in &centred {
                let projection = dot(row, &direction);
                for (sum, value) in next.iter_mut().zip(row) {
                    *sum += projection * value;
                }
            }
            for previous in &axes {
                let overlap = dot(&next, previous);
                for (value, along) in next.iter_mut().zip(previous) {
                    *value -= overlap * along;
                }
            }
            let norm = dot(&next, &next).sqrt();
            if norm <= f32::EPSILON {
                break;
            }
            direction = next.into_iter().map(|value| value / norm).collect();
        }
        let largest = direction
            .iter()
            .copied()
            .max_by(|a, b| a.abs().total_cmp(&b.abs()))
            .unwrap_or(1.0);
        if largest < 0.0 {
            direction.iter_mut().for_each(|value| *value = -*value);
        }
        axes.push(direction);
    }
    centred
        .iter()
        .map(|row| [dot(row, &axes[0]), dot(row, &axes[1])])
        .collect()
}

/// A position from an id alone, for jitter: the same entry always gets it.
fn jitter(entry_id: i64, radius: f32) -> [f32; 2] {
    let hash = vector_hash(&[entry_id as f32, (entry_id >> 24) as f32]);
    let angle = (hash % 3_600) as f32 / 3_600.0 * std::f32::consts::TAU;
    [radius * angle.cos(), radius * angle.sin()]
}

/// The force simulation. `start` holds every simulated point's first position;
/// `anchored[i]` holds a placed point's saved position, which it may leave by at
/// most `max_shift`. Others move freely. Units: the ideal spacing is 1.
fn simulate(
    start: Vec<[f32; 2]>,
    graph: &[Vec<(usize, f64)>],
    group_of: &[Option<usize>],
    anchored: &[Option<[f32; 2]>],
    iterations: usize,
    max_shift: f32,
) -> Vec<[f32; 2]> {
    let n = start.len();
    let mut points = start;
    if n < 2 {
        return points;
    }
    const CUTOFF: f32 = 4.0;
    let groups = group_of.iter().flatten().max().map_or(0, |last| last + 1);
    let extent = (n as f32).sqrt();
    let settling = anchored.iter().any(Option::is_some);
    let first_step = if settling { 0.5 } else { extent * 0.25 };
    let mut forces = vec![[0.0_f32; 2]; n];
    let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for iteration in 0..iterations {
        let step = first_step * (1.0 - iteration as f32 / iterations as f32) + 0.01;
        forces.iter_mut().for_each(|force| *force = [0.0, 0.0]);
        // Repulsion between points closer than the cutoff, found through a grid.
        cells.clear();
        let cell_of = |point: [f32; 2]| {
            (
                (point[0] / CUTOFF).floor() as i32,
                (point[1] / CUTOFF).floor() as i32,
            )
        };
        for (index, point) in points.iter().enumerate() {
            cells.entry(cell_of(*point)).or_default().push(index);
        }
        for i in 0..n {
            let (cx, cy) = cell_of(points[i]);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let Some(others) = cells.get(&(cx + dx, cy + dy)) else {
                        continue;
                    };
                    for &j in others {
                        if j == i {
                            continue;
                        }
                        let mut delta = [points[i][0] - points[j][0], points[i][1] - points[j][1]];
                        let mut distance = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
                        if distance < 1e-4 {
                            // Two points on one spot: part them the same way every time.
                            delta = jitter((i * n + j) as i64, 1.0);
                            distance = 1.0;
                        }
                        if distance >= CUTOFF {
                            continue;
                        }
                        let push = 1.0 / distance.max(0.05);
                        forces[i][0] += delta[0] / distance * push;
                        forces[i][1] += delta[1] / distance * push;
                    }
                }
            }
        }
        // Springs: alike entries want to be close, the more alike the closer.
        for (i, row) in graph.iter().enumerate() {
            for (j, weight) in row {
                let similarity = (*weight / f64::from(1.0 - KNOWLEDGE_MAP_GROUP_BASELINE))
                    .clamp(0.0, 1.0) as f32;
                let rest = 0.6 + 2.4 * (1.0 - similarity);
                let delta = [points[*j][0] - points[i][0], points[*j][1] - points[i][1]];
                let distance = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt().max(1e-4);
                let pull = (distance - rest) * (0.1 + 0.6 * similarity);
                forces[i][0] += delta[0] / distance * pull;
                forces[i][1] += delta[1] / distance * pull;
            }
        }
        // A pull to the group's centre, and a weak one to the middle so parts
        // that share nothing do not drift apart for ever.
        let mut centres = vec![[0.0_f32; 3]; groups];
        for (i, group) in group_of.iter().enumerate() {
            if let Some(group) = group {
                centres[*group][0] += points[i][0];
                centres[*group][1] += points[i][1];
                centres[*group][2] += 1.0;
            }
        }
        for (i, group) in group_of.iter().enumerate() {
            if let Some(group) = group {
                let centre = centres[*group];
                forces[i][0] += (centre[0] / centre[2] - points[i][0]) * 0.08;
                forces[i][1] += (centre[1] / centre[2] - points[i][1]) * 0.08;
            }
            forces[i][0] -= points[i][0] * 0.01;
            forces[i][1] -= points[i][1] * 0.01;
        }
        for i in 0..n {
            let [fx, fy] = forces[i];
            let magnitude = (fx * fx + fy * fy).sqrt();
            if !magnitude.is_finite() || magnitude <= 0.0 {
                continue;
            }
            let mut length = magnitude.min(step);
            if anchored[i].is_some() {
                length *= 0.1;
            }
            let mut next = [
                points[i][0] + fx / magnitude * length,
                points[i][1] + fy / magnitude * length,
            ];
            if let Some(saved) = anchored[i] {
                let away = [next[0] - saved[0], next[1] - saved[1]];
                let distance = (away[0] * away[0] + away[1] * away[1]).sqrt();
                if distance > max_shift {
                    next = [
                        saved[0] + away[0] / distance * max_shift,
                        saved[1] + away[1] / distance * max_shift,
                    ];
                }
            }
            if next[0].is_finite() && next[1].is_finite() {
                points[i] = next;
            }
        }
    }
    points
}

fn bounds(points: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    if points.is_empty() {
        return ([0.0, 0.0], [0.0, 0.0]);
    }
    let mut min = [f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN];
    for point in points {
        for axis in 0..2 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    (min, max)
}

/// Convex hull by the monotone chain, counter-clockwise; fewer than three
/// distinct points come back as they are.
pub(crate) fn convex_hull(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    sorted.dedup();
    if sorted.len() < 3 {
        return sorted;
    }
    let cross = |o: [f32; 2], a: [f32; 2], b: [f32; 2]| {
        (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut hull: Vec<[f32; 2]> = Vec::with_capacity(sorted.len() * 2);
    for pass in 0..2 {
        let start = hull.len();
        let ordered: Vec<[f32; 2]> = if pass == 0 {
            sorted.clone()
        } else {
            sorted.iter().rev().copied().collect()
        };
        for point in ordered {
            while hull.len() >= start + 2
                && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
            {
                hull.pop();
            }
            hull.push(point);
        }
        hull.pop();
    }
    hull
}

/// The order titles are tried in: most used first, then the lower id.
pub(crate) fn map_label_order(entries: &[KnowledgeEntry]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by(|left, right| {
        let uses = |index: usize| entries[index].reference_count + entries[index].recurrences;
        uses(*right)
            .cmp(&uses(*left))
            .then(entries[*left].id.cmp(&entries[*right].id))
    });
    order
}

/// The map for `entries`, joined with `vectors` by id. `saved` decides how it is
/// placed (spec Req 5): every entry placed by a matching fingerprint → reused;
/// some → settled around them; none → laid out afresh. `previous`, when it was
/// made from the same ids and vectors, lends its neighbours, groups and points.
pub(crate) fn compute_map(
    entries: &[KnowledgeEntry],
    vectors: &[(i64, Vec<f32>)],
    saved: &[SavedPosition],
    previous: Option<&KnowledgeMap>,
    now: u64,
) -> MapOutcome {
    let by_id: HashMap<i64, &Vec<f32>> = vectors.iter().map(|(id, vector)| (*id, vector)).collect();
    let entry_ids: Vec<i64> = entries.iter().map(|entry| entry.id).collect();
    // A vector holding NaN or infinity (a corrupt blob) counts as no vector: the
    // entry goes on the outer ring rather than to a non-finite point.
    let vectors: Vec<Option<Vec<f32>>> = entries
        .iter()
        .map(|entry| {
            by_id
                .get(&entry.id)
                .filter(|vector| vector.iter().all(|value| value.is_finite()))
                .map(|vector| normalised(vector))
        })
        .collect();
    let fingerprints: Vec<Option<u64>> = entries
        .iter()
        .map(|entry| by_id.get(&entry.id).map(|vector| vector_hash(vector)))
        .collect();
    if let Some(previous) = previous
        .filter(|previous| previous.entry_ids == entry_ids && previous.fingerprints == fingerprints)
    {
        let mut map = previous.clone();
        map.label_order = map_label_order(entries);
        for group in &mut map.groups {
            group.freshness = mean_freshness(entries, &group.members, now);
        }
        map.placement = MapPlacement::Reused;
        return MapOutcome { map, save: None };
    }

    let neighbours = map_neighbours(&vectors, KNOWLEDGE_MAP_NEIGHBOURS);
    let edges = map_edges(&neighbours);
    let group_of = map_groups(&neighbours);
    let graph = weighted_graph(&neighbours);
    let saved_by_id: HashMap<i64, &SavedPosition> = saved
        .iter()
        .map(|position| (position.entry_id, position))
        .collect();
    let placed: Vec<Option<[f32; 2]>> = entries
        .iter()
        .zip(&fingerprints)
        .map(|(entry, fingerprint)| {
            let position = saved_by_id.get(&entry.id)?;
            (Some(position.vector_hash) == *fingerprint).then_some([position.x, position.y])
        })
        .collect();
    let simulated: Vec<usize> = (0..entries.len())
        .filter(|i| vectors[*i].is_some())
        .collect();
    let placed_count = simulated.iter().filter(|i| placed[**i].is_some()).count();
    let placement = if simulated.is_empty() || placed_count == 0 {
        MapPlacement::Fresh
    } else if placed_count == simulated.len() {
        MapPlacement::Reused
    } else {
        MapPlacement::Settled
    };

    let mut points = vec![[0.0_f32, 0.0]; entries.len()];
    match placement {
        MapPlacement::Reused => {
            for i in &simulated {
                points[*i] = placed[*i].unwrap_or_default();
            }
        }
        MapPlacement::Fresh | MapPlacement::Settled => {
            let local: HashMap<usize, usize> = simulated
                .iter()
                .enumerate()
                .map(|(local, global)| (*global, local))
                .collect();
            let local_graph: Vec<Vec<(usize, f64)>> = simulated
                .iter()
                .map(|global| {
                    graph[*global]
                        .iter()
                        .filter_map(|(other, weight)| {
                            local.get(other).map(|other| (*other, *weight))
                        })
                        .collect()
                })
                .collect();
            let local_groups: Vec<Option<usize>> =
                simulated.iter().map(|global| group_of[*global]).collect();
            let (start, anchored, iterations, max_shift) = if placement == MapPlacement::Fresh {
                let refs: Vec<&[f32]> = simulated
                    .iter()
                    .filter_map(|i| vectors[*i].as_deref())
                    .collect();
                let plane = principal_plane(&refs);
                let (min, max) = bounds(&plane);
                let span = (max[0] - min[0]).max(max[1] - min[1]).max(1e-6);
                let scale = (simulated.len() as f32).sqrt() * 1.5 / span;
                let start = plane.iter().map(|[x, y]| [x * scale, y * scale]).collect();
                (
                    start,
                    vec![None; simulated.len()],
                    KNOWLEDGE_MAP_ITERATIONS,
                    0.0,
                )
            } else {
                let anchors: Vec<[f32; 2]> = simulated.iter().filter_map(|i| placed[*i]).collect();
                let (min, max) = bounds(&anchors);
                let extent = (max[0] - min[0]).max(max[1] - min[1]).max(1.0);
                let centre = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
                let start = simulated
                    .iter()
                    .map(|global| {
                        if let Some(position) = placed[*global] {
                            return position;
                        }
                        // A new entry starts at its placed neighbours' weighted centre.
                        let mut sum = [0.0_f32; 3];
                        for (other, similarity) in &neighbours[*global] {
                            if let Some(position) = placed[*other] {
                                let weight = similarity.max(0.0);
                                sum[0] += position[0] * weight;
                                sum[1] += position[1] * weight;
                                sum[2] += weight;
                            }
                        }
                        let id = entries[*global].id;
                        if sum[2] > 0.0 {
                            let offset = jitter(id, 0.3);
                            [sum[0] / sum[2] + offset[0], sum[1] / sum[2] + offset[1]]
                        } else {
                            let offset = jitter(id, extent / 2.0 + 1.0);
                            [centre[0] + offset[0], centre[1] + offset[1]]
                        }
                    })
                    .collect();
                let anchored = simulated.iter().map(|global| placed[*global]).collect();
                (
                    start,
                    anchored,
                    KNOWLEDGE_MAP_SETTLE_ITERATIONS,
                    extent * KNOWLEDGE_MAP_SETTLE_MAX_SHIFT,
                )
            };
            let laid = simulate(
                start,
                &local_graph,
                &local_groups,
                &anchored,
                iterations,
                max_shift,
            );
            for (local, global) in simulated.iter().enumerate() {
                points[*global] = laid[local];
            }
        }
    }

    // Entries without a vector: on a ring outside everything else.
    let lone: Vec<usize> = (0..entries.len())
        .filter(|i| vectors[*i].is_none())
        .collect();
    if !lone.is_empty() {
        let placed_points: Vec<[f32; 2]> = simulated.iter().map(|i| points[*i]).collect();
        let (min, max) = bounds(&placed_points);
        let centre = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
        let radius = ((max[0] - min[0]).hypot(max[1] - min[1]) / 2.0) + 2.0;
        for (k, i) in lone.iter().enumerate() {
            let angle = k as f32 / lone.len() as f32 * std::f32::consts::TAU;
            points[*i] = [
                centre[0] + radius * angle.cos(),
                centre[1] + radius * angle.sin(),
            ];
        }
    }

    let members = central_members(&group_of, &neighbours, &entry_ids);
    let groups = members
        .into_iter()
        .map(|members| {
            let positions: Vec<[f32; 2]> = members.iter().map(|i| points[*i]).collect();
            MapGroup {
                name: map_group_name(&entries[members[0]].title),
                hull: convex_hull(&positions),
                freshness: mean_freshness(entries, &members, now),
                members,
            }
        })
        .collect();
    let mut linked = vec![Vec::new(); entries.len()];
    for (a, b) in &edges {
        linked[*a].push(*b);
        linked[*b].push(*a);
    }
    let (min, max) = bounds(&points);
    let save = (placement != MapPlacement::Reused).then(|| {
        simulated
            .iter()
            // A non-finite point (a stored vector holding NaN) is not saved:
            // the column is NOT NULL, and one bad row would fail every save.
            .filter(|i| points[**i].iter().all(|value| value.is_finite()))
            .filter_map(|i| {
                Some(SavedPosition {
                    entry_id: entries[*i].id,
                    x: points[*i][0],
                    y: points[*i][1],
                    vector_hash: fingerprints[*i]?,
                })
            })
            .collect()
    });
    MapOutcome {
        map: KnowledgeMap {
            entry_ids,
            fingerprints,
            points,
            group_of,
            edges,
            linked,
            groups,
            label_order: map_label_order(entries),
            min,
            max,
            placement,
        },
        save,
    }
}

fn mean_freshness(entries: &[KnowledgeEntry], members: &[usize], now: u64) -> f64 {
    if members.is_empty() {
        return 0.0;
    }
    members
        .iter()
        .map(|index| entries[*index].freshness(now))
        .sum::<f64>()
        / members.len() as f64
}
