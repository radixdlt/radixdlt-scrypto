use super::utils::OverlayingIterator;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

#[test]
fn overlaying_iterator_overlays_changes() {
    let underlying = vec![(1, "cat"), (2, "dog"), (5, "ant"), (7, "bat")].into_iter();
    let overlaid = vec![
        (0, Some("bee")),  // add element before all existing ones
        (2, None),         // delete some existing element
        (3, Some("rat")),  // add element between some existing ones
        (4, None),         // delete some non-existing element
        (5, Some("cow")),  // replace some element's value
        (10, Some("fox")), // add element after all existing ones
    ]
    .into_iter();
    let overlying = OverlayingIterator::new(underlying, overlaid);
    assert_eq!(
        overlying.collect::<Vec<_>>(),
        vec![
            (0, "bee"),
            (1, "cat"),
            (3, "rat"),
            (5, "cow"),
            (7, "bat"),
            (10, "fox")
        ]
    );
}

#[test]
fn overlaying_iterator_returns_underlying_when_no_changes() {
    let underlying = vec![(1, "cat"), (2, "dog"), (5, "ant"), (7, "bat")].into_iter();
    let overlaid = vec![].into_iter();
    let overlying = OverlayingIterator::new(underlying, overlaid);
    assert_eq!(
        overlying.collect::<Vec<_>>(),
        vec![(1, "cat"), (2, "dog"), (5, "ant"), (7, "bat")]
    );
}

#[test]
fn overlaying_iterator_returns_upserted_values_when_no_underlying() {
    let underlying = vec![].into_iter();
    let overlaid = vec![(0, Some("bee")), (2, None), (5, Some("ant")), (7, None)].into_iter();
    let overlying = OverlayingIterator::new(underlying, overlaid);
    assert_eq!(overlying.collect::<Vec<_>>(), vec![(0, "bee"), (5, "ant")]);
}
