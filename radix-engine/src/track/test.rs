use radix_common::prelude::*;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

#[test]
fn overlaying_iterator_overlays_changes() {
    let underlying = vec![(1, "cat"), (2, "dog"), (5, "ant"), (7, "bat")].into_iter();
    let overlaying = vec![
        (0, Some("bee")),  // add element before all existing ones
        (2, None),         // delete some existing element
        (3, Some("rat")),  // add element between some existing ones
        (4, None),         // delete some non-existing element
        (5, Some("cow")),  // replace some element's value
        (10, Some("fox")), // add element after all existing ones
    ]
    .into_iter();
    let overlaying = OverlayingIterator::new(underlying, overlaying);
    assert_eq!(
        overlaying.collect::<Vec<_>>(),
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
    let overlaying = vec![].into_iter();
    let overlaying = OverlayingIterator::new(underlying, overlaying);
    assert_eq!(
        overlaying.collect::<Vec<_>>(),
        vec![(1, "cat"), (2, "dog"), (5, "ant"), (7, "bat")]
    );
}

#[test]
fn overlaying_iterator_returns_upserted_values_when_no_underlying() {
    let underlying = vec![].into_iter();
    let overlaying = vec![(0, Some("bee")), (2, None), (5, Some("ant")), (7, None)].into_iter();
    let overlaying = OverlayingIterator::new(underlying, overlaying);
    assert_eq!(overlaying.collect::<Vec<_>>(), vec![(0, "bee"), (5, "ant")]);
}

#[test]
fn overlaying_result_iterator_overlays_changes() {
    let underlying = vec![
        Result::<_, ()>::Ok((1, "cat")),
        Ok((2, "dog")),
        Ok((5, "ant")),
        Ok((7, "bat")),
    ]
    .into_iter();
    let overlaying = vec![
        (0, Some("bee")),  // add element before all existing ones
        (2, None),         // delete some existing element
        (3, Some("rat")),  // add element between some existing ones
        (4, None),         // delete some non-existing element
        (5, Some("cow")),  // replace some element's value
        (10, Some("fox")), // add element after all existing ones
    ]
    .into_iter();
    let overlaying = OverlayingResultIterator::new(underlying, overlaying);
    assert_eq!(
        overlaying.collect::<Vec<_>>(),
        vec![
            Ok((0, "bee")),
            Ok((1, "cat")),
            Ok((3, "rat")),
            Ok((5, "cow")),
            Ok((7, "bat")),
            Ok((10, "fox"))
        ]
    );
}

#[test]
fn overlaying_result_iterator_returns_underlying_when_no_changes() {
    let underlying = vec![
        Result::<_, ()>::Ok((1, "cat")),
        Ok((2, "dog")),
        Ok((5, "ant")),
        Ok((7, "bat")),
    ]
    .into_iter();
    let overlaying = vec![].into_iter();
    let overlaying = OverlayingResultIterator::new(underlying, overlaying);
    assert_eq!(
        overlaying.collect::<Vec<_>>(),
        vec![
            Ok((1, "cat")),
            Ok((2, "dog")),
            Ok((5, "ant")),
            Ok((7, "bat"))
        ]
    );
}

#[test]
fn overlaying_result_iterator_returns_upserted_values_when_no_underlying() {
    let underlying = Vec::<Result<_, ()>>::new().into_iter();
    let overlaying = vec![(0, Some("bee")), (2, None), (5, Some("ant")), (7, None)].into_iter();
    let overlaying = OverlayingResultIterator::new(underlying, overlaying);
    assert_eq!(
        overlaying.collect::<Vec<_>>(),
        vec![Ok((0, "bee")), Ok((5, "ant"))]
    );
}

#[test]
fn underlying_error_returns_error() {
    let underlying = vec![
        Result::<_, ()>::Ok((1, "cat")),
        Ok((2, "dog")),
        Err(()),
        Ok((7, "bat")),
    ]
    .into_iter();
    let overlaying = vec![
        (0, Some("bee")),  // add element before all existing ones
        (2, None),         // delete some existing element
        (3, Some("rat")),  // add element between some existing ones
        (4, None),         // delete some non-existing element
        (5, Some("cow")),  // replace some element's value
        (10, Some("fox")), // add element after all existing ones
    ]
    .into_iter();
    let overlaying = OverlayingResultIterator::new(underlying, overlaying);
    assert_eq!(
        overlaying.collect::<Vec<_>>(),
        vec![Ok((0, "bee")), Ok((1, "cat")), Err(()),]
    );
}
