use nom_7 as nom;

fn read_alpha1<T>(s: T) -> nom::IResult<T, T>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    nom::character::streaming::alpha1(s)
}

fn read_alpha1_final<T>(s: T) -> nom::IResult<T, T>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar,
{
    nom::character::complete::alpha1(s)
}

fn read_ws1<T>(s: T) -> nom::IResult<T, T>
where
    T: nom::InputTakeAtPosition,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar + core::clone::Clone,
{
    nom::character::streaming::space1(s)
}

#[test]
fn test_nom_string() {
    let s = crate::StringData::from_static("hello world");
    let (s, res) = read_alpha1(s).unwrap();
    assert_eq!(res, "hello");
    let (s, res) = read_ws1(s).unwrap();
    assert_eq!(res, " ");
    let (s, res) = read_alpha1_final(s).unwrap();
    assert_eq!(res, "world");
    assert_eq!(s, "")
}

#[cfg(feature = "queue")]
#[test]
fn test_nom_string_queue() {
    let s = crate::queue::StringQueue::from_iter([
        crate::StringData::from_static("hel"),
        crate::StringData::from_static("lo"),
        crate::StringData::from_static(" w"),
        crate::StringData::from_static("orld"),
    ]);
    let (s, res) = read_alpha1(s).unwrap();
    assert_eq!(res, "hello");
    let (s, res) = read_ws1(s).unwrap();
    assert_eq!(res, " ");
    let (s, res) = read_alpha1_final(s).unwrap();
    assert_eq!(res, "world");
    assert_eq!(s, "")
}
