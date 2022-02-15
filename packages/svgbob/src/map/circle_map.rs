use crate::{
    buffer::{CellBuffer, Contacts, Span},
    fragment,
    fragment::Arc,
    fragment::Circle,
    Cell, Point, Settings,
};
use lazy_static::lazy_static;
use std::{collections::BTreeMap, collections::HashMap, iter::FromIterator};

#[derive(PartialEq, Debug, Clone, Copy)]
/// edge cases of the circle art
pub enum Horizontal {
    /// circle arc is touching the left edge of the first cell
    /// ie: if the left most cell is `/` then it is touching the dge
    LeftEdge,
    /// if the left most cell is `(` or `|` then it starts at half the cell
    Half,
}

pub enum Vertical {
    /// the top edge of the cell
    TopEdge,
    /// the vertical middle of the cell
    Middle,
    /// the bottom edge of the cell
    BottomEdge,
}

// These are circle map, when a group is detected to have these set of characters
// arrange together in such this way, then endorse them as a circle
// Each of these character formation will have a certain circle parameters: center, and radius.
//
lazy_static! {

    /// ```ignore
    ///      0 1 2 3 4           B C D
    ///     0┌─┬─┬─┬─┐        A┌─┬─┬─┬─┐E
    ///     1├─┼─┼─┼─┤         │ │ │ │ │
    ///     2├─┼─┼─┼─┤        F├─G─H─I─┤J
    ///     3├─┼─┼─┼─┤         │ │ │ │ │
    ///     4├─┼─┼─┼─┤        K├─L─M─N─┤O
    ///     5├─┼─┼─┼─┤         │ │ │ │ │
    ///     6├─┼─┼─┼─┤        P├─Q─R─S─┤T
    ///     7├─┼─┼─┼─┤         │ │ │ │ │
    ///     8└─┴─┴─┴─┘        U└─┴─┴─┴─┘Y
    /// ```                      V W X
    /// art, edge_case, top_offset to center point, and the center cell in arc2
    static ref CIRCLE_ART_MAP: Vec<(&'static str, Horizontal,  f32, Cell)> =
        vec![
            // CIRCLE_1
            //center 1,0,k, radius = 0.5
            // 2 cell width , radius formula: (n-1)/2 = (2-1)/2 = 0.5
            // where n is the number of cells used
            //  if edge_case starts at edge then n is added by 1.
            //  vert_mid: half (0.5/1.0)
            //  cx_lies: edge
            //  cy_lies: mid
            //
            //  edge_case: start_half = 0.5, start_edge = 0.0
            //  if radius +  edge_case has 0.5 then mid, 0.0 then edge
            //
            //
            (r#"
            ()
            "#, Horizontal::Half,  0.5, Cell::new(0,0)),

            // CIRCLE_2
            //center = 1,1,m radius = 1.0
            // 3 cell width, (n-1)/2 = (3-1)/2  = 1.0
            // vert_mid: half  (0.5/1.0)
            // cx_lies: mid
            // cy_lies: mid
            (r#"
            (_)
            "#, Horizontal::Half,  0.5, Cell::new(1,0)),

            // CIRCLE_3
            //center = 1,1,o radius = 1.5,
            // 4 cell width, (n-1)/2 = (4-1)/2 = 1.5
            // vert_mid: 3/4 (1.5/2.0)
            // cx_lies: edge
            // cy_lies: mid
            (r#"
             __
            (__)
            "#,  Horizontal::Half, 1.5, Cell::new(1,1)),

            // CIRCLE_4
            //center: 2,1,m radius: 2.0
            //  5 cell width, (n-1)/2 = (5-1)/2 = 2.0
            //  vert_mid: half (1.5/3.0)
            // cx_lies: mid
            // cy_lies: mid
            (r#"
             ,-.
            (   )
             `-'
            "#,  Horizontal::Half,  1.5, Cell::new(2,1)),

            // CIRCLE_5
            //center: 2,1,o radius: 2.5
            // 6 cell width, (n-1)/2 = (6-1)/2 = 2.5
            // vert_mid: half (1.5/3.0)
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
             .--.
            (    )
             `--'
            "#, Horizontal::Half,  1.5, Cell::new(2,1)),

            // CIRCLE_6
            //center: 3,2,m radius: 3.0
            // 7 cell width, (n-1)/2 = (7-1)/2 = 3.0
            // vert_mid: 2.5/4
            // cx_lies: mid
            // cy_lies: mid
            (r#"
               _
             .' '.
            (     )
             `._.'
            "#, Horizontal::Half,  2.5, Cell::new(3,2)),

            // CIRCLE_7
            //center: 3,2,o radius: 3.5
            // 8 cell width, (n-1)/2 = (8-1)/2 = 3.5
            // vert_mid: 2.5/4
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
               __
             ,'  '.
            (      )
             `.__.'
            "#, Horizontal::Half,  2.5, Cell::new(3,2)),

            // CIRCLE_8
            //center: 4,2,m radius:4.0
            // 9 cell width, (n-1)/2 = (9-1)/2 = 4.0
            // vert_mid: half (2.5/5.0)
            // cx_lies: mid
            // cy_lies: mid
            (r#"
               ___
             ,'   '.
            (       )
             `.   .'
               `-'
            "#, Horizontal::Half,  2.5, Cell::new(4,2)),

            // circle 9 and up can be divided into 4 quadrants these quadrants can be arcs and can be used as
            // rounded edge with larger radius for rouded rect
            // CIRCLE_9
            //center: 4,2,w radius: 4.5
            // start_edge: true
            // 9 cell width, (n-0)/2 = (9-0)/2 = 4.5
            // vert_mid:  3.0/5.0
            //  cx_lies: mid
            //  cy_lies: edge
            (r#"
               ___
             ,'   `.
            /       \
            \       /
             `.___.'
            "#,  Horizontal::LeftEdge,  3.0, Cell::new(4,2) ),

            // CIRCLE_10
            //center: 4,2,y radius: 5.0
            //start_edge: true
            // 10 cell width, (n-0)/2 = (10-0)/2 = 5.0
            // vert_mid:  3.0/5.0
            //  cx_lies: edge
            //  cy_lies: edge
            (r#"
               ____
             ,'    `.
            /        \
            \        /
             `.____.'
            "#,  Horizontal::LeftEdge, 3.0, Cell::new(4,2)),

            // CIRCLE_11
            //center:5,3,o radius: 5.5
            // 12 cell width, (n-1)/2 = (12-1)/2 = 5.5
            // vert_mid:  3.5/6.0
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
                ____
              .'    `.
             /        \
            (          )
             \        /
              `.____.'
            "#, Horizontal::Half ,  3.5, Cell::new(5,3)),

            // CIRCLE_12
            //center:6,3,m radius: 6.0
            // 13 cell width, (n-1)/2 = (13-1)/2 = 6.0
            // vert_mid: 3.5/6.0
            // cx_lies: mid
            // cy_lies: mid
            (r#"
                _____
              ,'     `.
             /         \
            (           )
             \         /
              `._____.'
            "#, Horizontal::Half,  3.5, Cell::new(6,3)),

            // CIRCLE_13
            // center: 6,3,y radius: 6.5
            // vert_mid: 4.0/7.0
            //  cx_lies: edge
            //  cy_lies: edge
            (r#"
                ______
              ,'      `.
             /          \
            |            |
            |            |
             \          /
              `.______.'
            "#, Horizontal::Half,   4.0, Cell::new(6,3)),

            // CIRCLE_14
            //center: 7,3,w radius: 7.0
            //vert_mid: 4.0/7.0
            //  cx_lies: mid
            //  cy_lies: edge
            (r#"
                _______
              ,'       `.
             /           \
            |             |
            |             |
             \           /
              `._______.'
            "#, Horizontal::Half ,  4.0, Cell::new(7,3)),


            // CIRCLE_15
            //center: 7,4,o radius: 7.5
            //vert_mid: 4.5/8.0
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
                ________
              ,'        `.
             /            \
            |              |
            |              |
            |              |
             \            /
              `.________.'
            "#, Horizontal::Half,  4.5, Cell::new(7,4)),

            // CIRCLE_16
            //center: 8,4,m radius: 8.0
            //vert_mid: 4.5/9.0
            // cx_lies: mid
            // cy_lies: mid
            (r#"
                __-----__
              ,'         `.
             /             \
            |               |
            |               |
            |               |
             \             /
              `.         .'
                `-------'
            "#, Horizontal::Half,  4.5, Cell::new(8,4)),

            // CIRCLE_17
            //center: 8,4,o radius: 8.5
            // vert_mid:  4.5/9.0
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
                .--------.
              ,'          `.
             /              \
            |                |
            |                |
            |                |
             \              /
              `.          .'
                `--------'
            "#, Horizontal::Half,  4.5, Cell::new(8,4)),

            // CIRCLE_18
            //center:9,5,m radius: 9.0
            //vert_mid: 5.5/10.0
            // cx_lies: mid
            // cy_lies: mid
            (r#"
                _.-'''''-._
              ,'           `.
             /               \
            .                 .
            |                 |
            |                 |
            |                 |
             \               /
              `._         _.'
                 '-.....-'
            "#, Horizontal::Half,  5.5, Cell::new(9,5)),

            // CIRCLE_19
            // center: 9,5,o radius: 9.5
            // vert_mid:  5.5/10.0
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
                _.-''''''-._
              ,'            `.
             /                \
            .                  .
            |                  |
            |                  |
            |                  |
             \                /
              `._          _.'
                 '-......-'
            "#,  Horizontal::Half,  5.5, Cell::new(9,5)),


            // CIRCLE_20
            // center: 10,5,m radius: 10
            // vert_mid: 5.5/10.0
            // cx_lies: mid
            // cy_lies: mid
            (r#"
                _.-'''''''-._
              ,'             `.
             /                 \
            .                   .
            |                   |
            |                   |
            |                   |
             \                 /
              `._           _.'
                 '-.......-'
            "#, Horizontal::Half, 5.5, Cell::new(10,5)),

            // CIRCLE_21
            // center: 10,5,o radius: 10.5
            // vert_mid: 5.5/11.0
            //  cx_lies: edge
            //  cy_lies: mid
            (r#"
                _.-''''''''-._
              ,'              `.
             /                  \
            .                    .
            |                    |
            |                    |
            |                    |
            |                    |
             \                  /
              `._            _.'
                 '-........-'
            "#, Horizontal::Half,  5.5, Cell::new(10,5)),

            // CIRCLE_22
            // center: 10,5,m radius: 11
            // radius = (n-1)/2 = (23-1)/2 = 11
            // vert_mid: 5.5/11.0
            // cx_lies: mid
            // cy_lies: mid
            (r#"
                _.-'''''''''-._
              ,'               `.
             /                   \
            .                     .
            |                     |
            |                     |
            |                     |
            |                     |
             \                   /
              `._             _.'
                 '-.........-'
            "#, Horizontal::Half, 5.5, Cell::new(11, 5)),

        ];


    static ref CIRCLE_MAP: Vec<(&'static str, Point, f32, Horizontal, f32, Cell)> =Vec::from_iter(
        CIRCLE_ART_MAP.iter().enumerate().map(|(ndx, (art, edge_case, offset_center_y,arc2_center))|{
            let cb = CellBuffer::from(*art);
            let (lo, hi) = cb.bounds().expect("circle must have bounds");

           let width = match *edge_case {
               Horizontal::LeftEdge => {
                    (hi.x - lo.x) as f32 + 1.0
               }
               Horizontal::Half => {
                    (hi.x - lo.x) as f32
               }
           };

           println!("width: {}", width);
           let calc_radius = width / 2.0;
           println!("calc_radius: {}", calc_radius);
           let index = (width - 1.0) as usize;
           assert_eq!(ndx, index);

            let edge_inc_x = match edge_case{
                Horizontal::LeftEdge => 0.0,
                Horizontal::Half => 0.5,
            };

            let calc_center_x = calc_radius + edge_inc_x;
            let calc_center_y = offset_center_y * 2.0;
            let calc_center = Point::new(calc_center_x, calc_center_y);
            println!("center for circle: {} is {}", ndx, calc_center);

            (*art, calc_center, calc_radius, *edge_case, *offset_center_y, *arc2_center)
        })
    );


    /// The fragments for each of the circle
    /// Calculate the span and get the group fragments
    static ref FRAGMENTS_CIRCLE: Vec<(Vec<Contacts>,Circle)> = Vec::from_iter(
        CIRCLE_MAP.iter().map(|(art, center, radius, edge_case, offset_center_y, arc2_center)|{
            (circle_art_to_group(art, &Settings::default()), Circle::new(*center, *radius, false))
        })
    );

    /// map of circle spans and their radius
    pub static ref DIAMETER_CIRCLE: HashMap<i32,(Point,Span)> = HashMap::from_iter(
        CIRCLE_MAP.iter().map(|(art, center, radius, edge_case, offset_center_y, arc2_center)|{
            let cb = CellBuffer::from(*art);
            let mut spans = cb.group_adjacents();
            assert_eq!(spans.len(), 1);
            let span = spans.remove(0).localize();
            ((*radius * 2.0) as i32, (*center, span))
        })
    );

    /// There is only 1 span per circle, and localized
    pub static ref CIRCLES_SPAN: BTreeMap<Circle, Span> = BTreeMap::from_iter(
        CIRCLE_MAP.iter().map(|(art, center, radius, edge_case, offset_center_y, arc2_center)|{
            let cb = CellBuffer::from(*art);
            let mut spans = cb.group_adjacents();
            assert_eq!(spans.len(), 1);
            let span = spans.remove(0).localize();
            (Circle::new(*center, *radius, false), span)
        })
    );

    /// top_left      top       top_right
    ///               p2
    ///          arc2  |    arc1
    ///                |
    ///    left  p3----+----- p1 right
    ///                |
    ///          arc3  |   arc4
    ///               p4
    /// bottom_left  bottom   bottom_right
    ///
    /// (diameter, quarter arcs)
    pub static ref QUARTER_ARC_SPAN: BTreeMap<i32, [(Arc,Span);4]> = BTreeMap::from_iter(
        CIRCLE_MAP.iter().skip(3).map(|(art, center, radius, edge_case, offset_center_y, arc2_center)|{
            let span = circle_art_to_span(art);
            let bounds = span.cell_bounds().expect("must have bounds");
            let top_left = bounds.top_left();
            let bottom_right = bounds.bottom_right();
            let top_right = bounds.top_right();
            let bottom_left = bounds.bottom_left();

            let p1 = Point::new(center.x + radius, center.y);
            let p2 = Point::new(center.x, center.y - radius);
            let p3 = Point::new(center.x - radius, center.y);
            let p4 = Point::new(center.x, center.y + radius);

            println!("Circle no: {}", radius * 2.0);


            let span1_center = Cell::new((center.x.floor() / Cell::width()) as i32, arc2_center.y);
            let span2_center = *arc2_center;
            let span3_center = Cell::new(arc2_center.x, (center.y.floor() / Cell::height()) as i32);
            let span4_center = Cell::new((center.x.floor() / Cell::width()) as i32, (center.y.floor() / Cell::height()) as i32);


            let bounds1 = Cell::rearrange_bound(span1_center, top_right);
            let bounds2 = Cell::rearrange_bound(top_left, span2_center);
            let bounds3 = Cell::rearrange_bound(bottom_left, span3_center);
            let bounds4 = Cell::rearrange_bound(span4_center, bottom_right);

            let span1 = span.extract(bounds1.0, bounds1.1).localize();
            let span2 = span.extract(bounds2.0, bounds2.1).localize();
            let span3 = span.extract(bounds3.0, bounds3.1).localize();
            let span4 = span.extract(bounds4.0, bounds4.1).localize();

            println!("span1: \n{}\n", span1);
            println!("span2: \n{}\n", span2);
            println!("span3: \n{}\n", span3);
            println!("span4: \n{}\n", span4);

            let arc1_start  = bounds1.0.localize_point(p1);
            let arc1_end = bounds1.0.localize_point(p2);

            let arc2_start = bounds2.0.localize_point(p2);
            let arc2_end = bounds2.0.localize_point(p3);

            let arc3_start = bounds3.0.localize_point(p3);
            let arc3_end = bounds3.0.localize_point(p4);

            let arc4_start = bounds4.0.localize_point(p4);
            let arc4_end = bounds4.0.localize_point(p1);

            let arc1 = Arc::new(arc1_start, arc1_end, *radius);
            let arc2 = Arc::new(arc2_start, arc2_end, *radius);
            let arc3 = Arc::new(arc3_start, arc3_end, *radius);
            let arc4 = Arc::new(arc4_start, arc4_end, *radius);

            let diameter = (*radius * 2.0) as i32;
            (diameter, [(arc1, span1), (arc2, span2), (arc3, span3), (arc4, span4)])
        })
    );

    /*
    pub static ref HALF_ARC_SPAN: BTreeMap<i32,[(Arc,Span);4]> = BTreeMap::from_iter(
        QUARTER_ARC_SPAN.iter().map(|(diameter, [(arc1, span1), (arc2, span2), (arc3, span3), (arc4, span4)])|{
            let radius = (diameter / 2) as f32;

            let half12 = Arc::new(arc1.start, arc2.end, radius);
            let span12 = span2.paste_at(span2.cell_bounds().unwrap().top_right(), span1);

            let half23 = Arc::new(arc2.start, arc3.end, radius);
            let span23 = span2.paste_at(span2.cell_bounds().unwrap().bottom_left(), span3);

            let half34 = Arc::new(arc3.start, arc4.end, radius);
            let span34 = span3.paste_at(span3.cell_bounds().unwrap().top_right(), span4);

            let half41 = Arc::new(arc4.start, arc1.end, radius);
            let span41 = span1.paste_at(span1.cell_bounds().unwrap().bottom_left(), span4);

            (*diameter, [(half12, span12), (half23, span23), (half34, span34), (half41, span41)])
        })
    );
    */


    pub static ref ARC_SPAN: BTreeMap<DiameterArc, (Arc,Span)> = BTreeMap::from_iter(
            QUARTER_ARC_SPAN.iter().flat_map(|(diameter, arc_spans)|{
                arc_spans.iter().enumerate().map(move|(arc_index, arc_span)|{
                (
                DiameterArc{
                    diameter: *diameter,
                    arc: arc_index,
                },
                arc_span.clone()
                )
            })
        })
    );

}

#[derive(Default, Hash, PartialEq, PartialOrd, Eq)]
pub struct DiameterArc {
    diameter: i32,
    arc: usize,
}
impl Ord for DiameterArc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.diameter
            .cmp(&other.diameter)
            .then(self.arc.cmp(&other.arc))
    }
}

fn circle_art_to_group(art: &str, settings: &Settings) -> Vec<Contacts> {
    let span1 = circle_art_to_span(art);
    span1.get_contacts(settings)
}

fn circle_art_to_span(art: &str) -> Span {
    let cell_buffer = CellBuffer::from(art);
    let mut spans = cell_buffer.group_adjacents();
    assert_eq!(spans.len(), 1);
    spans.remove(0).localize()
}

pub fn endorse_circle_span(search: &Span) -> Option<(&Circle, Span)> {
    CIRCLES_SPAN.iter().rev().find_map(|(circle, span)| {
        let search_localized = search.clone().localize();
        let (matched, unmatched) = is_subset_of(span, &search_localized);
        if matched {
            let unmatched_cell_chars = search
                .iter()
                .enumerate()
                .filter_map(|(i, cell_char)| {
                    if unmatched.contains(&i) {
                        Some(cell_char.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            Some((circle, Span::from(unmatched_cell_chars)))
        } else {
            None
        }
    })
}

pub fn endorse_arc_span(search: &Span) -> Option<(&Arc, Span)> {
    ARC_SPAN.iter().rev().find_map(|(_diameter, (arc, span))| {
        let search_localized = search.clone().localize();
        let (matched, unmatched) = is_subset_of(span, &search_localized);
        if matched {
            let unmatched_cell_chars = search
                .iter()
                .enumerate()
                .filter_map(|(i, cell_char)| {
                    if unmatched.contains(&i) {
                        Some(cell_char.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            Some((arc, Span::from(unmatched_cell_chars)))
        } else {
            None
        }
    })
}

/// returns true if all the contacts in subset is in big_set
/// This also returns the indices of big_set that are not found in the subset
fn is_subset_of<T: PartialEq>(
    subset: &[T],
    big_set: &[T],
) -> (bool, Vec<usize>) {
    let mut unmatched = vec![];
    let mut matched = 0;
    for (_i, set) in subset.iter().enumerate() {
        if big_set.contains(set) {
            matched += 1;
        }
    }
    for (bi, bset) in big_set.iter().enumerate() {
        if !subset.contains(bset) {
            unmatched.push(bi);
        }
    }
    (matched == subset.len(), unmatched)
}

#[cfg(test)]
mod test_circle_map;