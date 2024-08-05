use nom_7 as nom;

fn read_alpha1<T>(st: T) -> nom::IResult<T, T>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    nom::character::streaming::alpha1(st)
}

fn read_alpha1_final<T>(st: T) -> nom::IResult<T, T>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    nom::character::complete::alpha1(st)
}

fn read_ws1<T>(st: T) -> nom::IResult<T, T>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar + core::clone::Clone,
{
    nom::character::streaming::space1(st)
}

#[test]
#[allow(clippy::unwrap_used, clippy::shadow_unrelated)]
fn test_nom_string() {
    let st = crate::StringData::from_static("hello world");
    let (st, res) = read_alpha1(st).unwrap();
    assert_eq!(res, "hello");
    let (st, res) = read_ws1(st).unwrap();
    assert_eq!(res, " ");
    let (st, res) = read_alpha1_final(st).unwrap();
    assert_eq!(res, "world");
    assert_eq!(st, "");
}

#[cfg(feature = "queue")]
#[test]
#[allow(clippy::unwrap_used, clippy::shadow_unrelated)]
fn test_nom_string_queue() {
    let st = crate::queue::StringQueue::from_iter([
        crate::StringData::from_static("hel"),
        crate::StringData::from_static("lo"),
        crate::StringData::from_static(" w"),
        crate::StringData::from_static("orld"),
    ]);
    let (st, res) = read_alpha1(st).unwrap();
    assert_eq!(res, "hello");
    let (st, res) = read_ws1(st).unwrap();
    assert_eq!(res, " ");
    let (st, res) = read_alpha1_final(st).unwrap();
    assert_eq!(res, "world");
    assert_eq!(st, "");
}
