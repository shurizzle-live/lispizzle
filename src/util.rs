use std::fmt;

pub fn print_list_debug<T>(
    f: &mut fmt::Formatter<'_>,
    iiter: impl IntoIterator<Item = T>,
    lh: impl fmt::Display,
    rh: impl fmt::Display,
) -> fmt::Result
where
    T: fmt::Debug,
{
    fmt::Display::fmt(&lh, f)?;

    let mut it = iiter.into_iter();
    if let Some(e) = it.next() {
        fmt::Debug::fmt(&e, f)?;

        for e in it {
            write!(f, " {:?}", e)?;
        }
    }

    fmt::Display::fmt(&rh, f)
}

pub fn print_list_display<T>(
    f: &mut fmt::Formatter<'_>,
    iiter: impl IntoIterator<Item = T>,
    lh: impl fmt::Display,
    rh: impl fmt::Display,
) -> fmt::Result
where
    T: fmt::Display,
{
    fmt::Display::fmt(&lh, f)?;

    let mut it = iiter.into_iter();
    if let Some(e) = it.next() {
        fmt::Display::fmt(&e, f)?;

        for e in it {
            write!(f, " {}", e)?;
        }
    }

    fmt::Display::fmt(&rh, f)
}
