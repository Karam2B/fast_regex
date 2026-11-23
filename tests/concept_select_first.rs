pub trait CapitilizeFirst {
    fn print_capital(&self, str: &mut String);
    fn print_small(&self, str: &mut String);
}

pub struct A;
pub struct B;

impl CapitilizeFirst for A {
    fn print_capital(&self, str: &mut String) {
        str.push('A');
    }
    fn print_small(&self, str: &mut String) {
        str.push('a');
    }
}

impl CapitilizeFirst for B {
    fn print_capital(&self, str: &mut String) {
        str.push('B');
    }
    fn print_small(&self, str: &mut String) {
        str.push('b');
    }
}

impl CapitilizeFirst for () {
    fn print_capital(&self, _: &mut String) {}
    fn print_small(&self, _: &mut String) {}
}

impl<T0> CapitilizeFirst for (T0,)
where
    T0: CapitilizeFirst,
{
    fn print_capital(&self, str: &mut String) {
        self.0.print_capital(str);
    }
    fn print_small(&self, str: &mut String) {
        self.0.print_small(str);
    }
}

impl<T0, T1> CapitilizeFirst for (T0, T1)
where
    T0: CapitilizeFirst,
    T1: CapitilizeFirst,
{
    fn print_capital(&self, str: &mut String) {
        self.0.print_capital(str);
        self.1.print_small(str);
    }
    fn print_small(&self, str: &mut String) {
        self.0.print_small(str);
        self.1.print_small(str);
    }
}

pub struct Nest<T>(pub T);

impl<N0, N1> CapitilizeFirst for (Nest<N0>, Nest<N1>)
where
    N0: CapitilizeFirst,
    N1: CapitilizeFirst,
{
    fn print_capital(&self, str: &mut String) {
        self.0.0.print_capital(str);
        self.1.0.print_small(str);
    }

    fn print_small(&self, str: &mut String) {
        self.0.0.print_small(str);
        self.1.0.print_small(str);
    }
}

impl CapitilizeFirst for Vec<Box<dyn CapitilizeFirst>> {
    fn print_capital(&self, str: &mut String) {
        let mut first_hit = false;
        for item in self {
            if !first_hit {
                item.print_capital(str);
                first_hit = true;
            } else {
                item.print_small(str);
            }
        }
    }

    fn print_small(&self, str: &mut String) {
        for item in self {
            item.print_small(str);
        }
    }
}

impl<T> CapitilizeFirst for Option<T>
where
    T: CapitilizeFirst,
{
    fn print_capital(&self, str: &mut String) {
        if let Some(item) = self {
            item.print_capital(str);
        }
    }
    fn print_small(&self, str: &mut String) {
        if let Some(item) = self {
            item.print_small(str);
        }
    }
}

#[test]
fn main() {
    let mut str = String::new();

    (A, B).print_capital(&mut str);

    assert_eq!(str, "Ab");

    // same
    str.clear();
    (Nest((A, B)), Nest(())).print_capital(&mut str);
    assert_eq!(str, "Ab");

    str.clear();

    (Nest((A, B)), Nest((A,))).print_capital(&mut str);

    assert_eq!(str, "Aba");

    str.clear();

    (Nest((A, B)), Nest((A, B))).print_capital(&mut str);
    // three levels of nesting

    assert_eq!(str, "Abab");

    str.clear();

    (Nest((Nest((A, B)), Nest((A, B)))), Nest((A,))).print_capital(&mut str);
    assert_eq!(str, "Ababa");

    str.clear();

    vec![
        Box::new(A) as Box<dyn CapitilizeFirst>,
        Box::new(B),
        Box::new((Nest((A, B)), Nest(()))),
        Box::new(B),
        Box::new(None::<A>),
        Box::new((A, B)),
        Box::new(Some(B)),
        Box::new(B),
    ]
    .print_capital(&mut str);

    assert_eq!(str, "Ababbabbb");
}

#[test]
fn test_no_nest() {
    let mut str = String::new();

    (A, B).print_capital(&mut str);

    assert_eq!(str, "Ab");

    // same
    str.clear();
    ((A, B), ()).print_capital(&mut str);
    assert_eq!(str, "Ab");

    str.clear();

    ((A, B), (A,)).print_capital(&mut str);

    assert_eq!(str, "Aba");

    str.clear();

    ((A, B), (A, B)).print_capital(&mut str);
    // three levels of nesting

    assert_eq!(str, "Abab");

    str.clear();

    (((A, B), (A, B)), (A,)).print_capital(&mut str);
    assert_eq!(str, "Ababa");

    str.clear();

    vec![
        Box::new(A) as Box<dyn CapitilizeFirst>,
        Box::new(B),
        Box::new(((A, B), ())),
        Box::new(B),
        Box::new(None::<A>),
        Box::new((A, B)),
        Box::new(Some(B)),
        Box::new(B),
    ]
    .print_capital(&mut str);

    assert_eq!(str, "Ababbabbb");
}

pub trait IntoBigger {
    type Bigger<T>;
    fn into_bigger<T>(self, next: T) -> Self::Bigger<T>;
}

impl IntoBigger for () {
    type Bigger<T> = (T,);
    fn into_bigger<T>(self, next: T) -> Self::Bigger<T> {
        (next,)
    }
}

impl<T0> IntoBigger for (T0,) {
    type Bigger<T1> = (T0, T1);
    fn into_bigger<T1>(self, next: T1) -> Self::Bigger<T1> {
        (self.0, next)
    }
}

impl<T0, T1> IntoBigger for (T0, T1) {
    type Bigger<T2> = ((T0, T1), T2);
    fn into_bigger<T2>(self, next: T2) -> Self::Bigger<T2> {
        ((self.0, self.1), next)
    }
}

#[test]
fn test_into_bigger() {
    let mut str = String::new();
    let s = ().into_bigger(A);

    s.print_capital(&mut str);
    assert_eq!(str, "A");
    str.clear();

    let s = s.into_bigger(B);
    s.print_capital(&mut str);
    assert_eq!(str, "Ab");
    str.clear();

    let s = s.into_bigger(A);
    s.print_capital(&mut str);
    assert_eq!(str, "Aba");
    str.clear();

    let s = s.into_bigger(B);
    s.print_capital(&mut str);
    assert_eq!(str, "Abab");
    str.clear();

    let s = s.into_bigger(A);
    s.print_capital(&mut str);
    assert_eq!(str, "Ababa");
    str.clear();

    let s = s.into_bigger(B);
    s.print_capital(&mut str);
    assert_eq!(str, "Ababab");
    str.clear();
}
